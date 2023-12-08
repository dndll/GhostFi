use axum::{
    extract::{Query, State},
    http::Method,
    routing::post,
    Json, Router,
};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::Any;
use tower_http::cors::CorsLayer;
use registrar::extract_passport_info;

pub mod config;
pub mod contract;
pub mod prover;

pub type Hash = [u8; 32];

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::debug!("starting up");

    let config = config::Config::new().expect("Failed to load config");
    log::debug!("config: {:#?}", config);

    let contract_client = Arc::new(contract::Client::new(&config));

    let cors = CorsLayer::permissive();

    let controller = Router::new()
        .route("/prove", post(prove).with_state(config.clone()))
        .route(
            "/prove/verify",
            post(prove_verify).with_state((contract_client.clone(), config.clone())),
        )
        .route(
            "/verify",
            post(verify).with_state((contract_client.clone(), config.clone())),
        )
        .layer(cors);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(controller.into_make_service())
        .await
        .unwrap();
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProofRequest {
    public_key: String,
    requested_amount: u64,
    params: Vec<Heuristic>,
}

const HEURISTIC_AMT: usize = 8;

impl ProofRequest {
    pub fn is_valid(&self) -> bool {
        // TODO: assert heuristic id's are unique
        true
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Heuristic {
    Simple { balance: u64 },
    Passport { country: String }
}

impl Into<u8> for Heuristic {
    fn into(self) -> u8 {
        let byte = match self {
            Heuristic::Simple { .. } => 1,
            Heuristic::Passport { .. } => 2,
        };
        assert!(
            byte <= HEURISTIC_AMT,
            "We only allow HEURISTIC_AMT heuristics in the circuit!"
        );
        byte as u8
    }
}

#[serde_with::serde_as]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Proof {
    public_key: String,
    requested_amount: u64,
    #[serde(with = "hex")]
    inner: Vec<u8>,
    account_id: Option<String>,
}

async fn prove(
    State(config): State<config::Config>,
    Json(req): Json<ProofRequest>,
) -> Json<Option<Proof>> {
    let cmd = prover::Command::Prove(req);
    Json(prover::execute(&config, cmd).ok())
}

async fn prove_verify(
    State((client, config)): State<(Arc<contract::Client>, config::Config)>,
    Json(req): Json<ProofRequest>,
) -> Json<bool> {
    let proof = prove(State(config.clone()), Json(req)).await.0;
    if let Some(proof) = proof {
        let no_submit = Submission { submit: false };
        verify(State((client, config)), Some(Query(no_submit)), Json(proof)).await
    } else {
        Json(false)
    }
}

#[derive(Deserialize)]
struct Submission {
    submit: bool,
}
impl Default for Submission {
    fn default() -> Self {
        Self { submit: true }
    }
}

pub struct VerificationResult(bool);

async fn verify(
    State((client, config)): State<(Arc<contract::Client>, config::Config)>,
    submission: Option<Query<Submission>>,
    Json(proof): Json<Proof>,
) -> Json<bool> {
    let cmd = prover::Command::Verify(proof.clone());
    let executed = prover::execute::<VerificationResult>(&config, cmd);
    let verified = match executed {
        Ok(res) => {
            if submission.unwrap_or_default().submit {
                client.verified(proof).await.unwrap_or_default()
            } else {
                res.0
            }
        }
        Err(e) => {
            log::error!("Error: {:?}", e);
            false
        }
    };
    Json(verified)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_deserialisation() {
        let json = std::fs::read_to_string("fixtures/proof.json").unwrap();
        let proof: Proof = serde_json::from_str(&json).unwrap();
        println!("{:?}", proof);
    }

    #[test]
    fn test_proof_request_deserialisation() {
        let json = std::fs::read_to_string("fixtures/simple.json").unwrap();
        let proof: ProofRequest = serde_json::from_str(&json).unwrap();
        println!("{:?}", proof);
    }

    #[test]
    fn test_passport() {
        let passport = extract_passport_info();
        println!("{:?}", passport);
    }
}
