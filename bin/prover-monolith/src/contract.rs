use std::str::FromStr;

use crate::{Proof, Result};
use near_account_id::AccountId;
use near_crypto::{InMemorySigner, PublicKey, SecretKey};
use serde_json::json;

#[derive(Clone)]
pub struct Client {
    contract: AccountId,
    client: near_fetch::Client,
    signer: InMemorySigner,
}

impl Client {
    pub fn new(config: &crate::config::Config) -> Self {
        let account_id: AccountId = config.account.parse().unwrap();
        log::info!("Using account {}", account_id);
        let sk = SecretKey::from_str(&config.secret).unwrap();
        log::debug!("Using sk {:?}", sk);
        let signer = near_crypto::InMemorySigner::from_secret_key(account_id, sk);

        let client = near_fetch::Client::new(&config.rpc);
        Self {
            signer,
            contract: config.account.parse().unwrap(),
            client,
        }
    }
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
    fn test_public_key_creation() {
        let config = crate::config::Config::from("../../local");
        let account_id: AccountId = config.account.parse().unwrap();
        let sk = SecretKey::from_str(&config.secret).unwrap();
        let pk = sk.public_key();

        println!("{}", pk);
        println!("{}", hex::encode(pk.key_data()));
        println!("{}", sk);
        let signer = near_crypto::InMemorySigner::from_secret_key(
            account_id,
            SecretKey::from_str(&config.secret).unwrap(),
        );
    }
}
