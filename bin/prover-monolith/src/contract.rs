use crate::{config::Config, Proof, Result};
use near_account_id::AccountId;
use near_crypto::{InMemorySigner, SecretKey};
use serde_json::json;
use std::str::FromStr;

#[derive(Clone)]
pub struct Client {
    contract: AccountId,
    client: near_fetch::Client,
    signer: InMemorySigner,
}

impl Client {
    pub fn new(config: &Config) -> Self {
        log::debug!("Initialising client, config {:?}", config);

        let signer = || -> Result<InMemorySigner> {
            Ok(near_crypto::InMemorySigner::from_secret_key(
                config.account.clone(),
                config.secret.clone(),
            ))
        };

        let client = near_fetch::Client::new(&config.rpc);
        Self {
            signer: signer().expect("Failed to create signer"),
            contract: config.account.parse().unwrap(),
            client,
        }
    }

    /// Interacts with the contract to inform the lender that the loan has been
    /// verified successfully
    pub async fn verified(&self, proof: Proof) -> Result<bool> {
        let account_id = proof
            .account_id
            .as_ref()
            .and_then(|x| x.parse::<AccountId>().ok())
            .ok_or_else(|| eyre::eyre!("Invalid account id"))?;

        let keys = self.client.view_access_keys(&account_id).await?;
        assert!(
            keys.keys.iter().any(|x| {
                log::debug!(
                    "Comparing pk {:?} with {:?}",
                    x.public_key,
                    proof.public_key
                );
                x.public_key.to_string() == proof.public_key
            }),
            "Key is not registered for this account"
        );

        let f = self
            .client
            .call(&self.signer, &self.contract, "verified_loan")
            .args_json(Self::build_verified_loan_call(proof).unwrap())
            .transact()
            .await;

        // TODO: get account from onchain for pk

        match f {
            Ok(_) => Ok(true),
            Err(err) => {
                log::error!("{}", err);
                Ok(false)
            }
        }
    }
    fn build_verified_loan_call(proof: Proof) -> Result<serde_json::Value> {
        let json = json!({
            "user": proof.account_id,
            "amount": format!("{}", proof.requested_amount)
        });
        log::debug!("Json {}", serde_json::to_string_pretty(&json)?);

        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() {
        let json = std::fs::read_to_string("fixtures/proof.json").unwrap();
        let proof: Proof = serde_json::from_str(&json).unwrap();
        println!(
            "{}",
            serde_json::to_string(&Client::build_verified_loan_call(proof).unwrap()).unwrap()
        );
    }


    #[test]
    fn test_sk() {
        let sk = SecretKey::from_random(near_crypto::KeyType::ED25519);
        println!("{}", sk.to_string());
        
    }
}
