use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use eyre::Result;
use near_account_id::AccountId;
use near_crypto::{InMemorySigner, KeyType};
use serde::{Deserialize, Serialize};

pub mod prover;

pub type Hash = [u8; 32];


#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::debug!("starting up");

    let account_id: AccountId = env!("GHOSTFI_ACCOUNT").parse().unwrap();
    let seed = env!("GHOSTFI_SECRET");
    let signer = near_crypto::InMemorySigner::from_seed(account_id, KeyType::ED25519, seed);

    let controller = Router::new()
        .route("/prove", post(proof))
//        .route("/prove/verify", post(proof_verify))
        .route("/verify", post(verify_proof).with_state(signer.clone()));

    // Block forever
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(controller.into_make_service())
        .await
        .unwrap();
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize)]
struct ProofRequest {
    #[serde(with = "hex")]
    public_key: [u8; 32],
    requested_amount: u128,
    params: Vec<Heuristic>,
}

#[derive(Serialize, Deserialize)]
pub struct Heuristic {
    // TODO: issue here because size of field is 127, we can allow hex tho
    params: Vec<u128>,
}

// TODO:
pub struct ProofData {}

// TODO:
pub type Proof = String;

async fn proof(Json(req): Json<ProofRequest>) -> Json<Option<Proof>> {
    // TODO: generate proof
    Json(None)
}

async fn verify_proof(
    State(signer): State<InMemorySigner>,
    Json(proof): Json<Proof>,
) -> Json<bool> {
    // TODO: verify proof
    // TODO: sign and submit to contract
    axum::Json(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_deserialisation() {}
}
