use eyre::Report;
use near_crypto::PublicKey;
use serde::{Deserialize, Serialize};
use tempfile::{tempfile, NamedTempFile, TempDir};

use crate::{config::Config, Heuristic, Proof, ProofRequest, Result, VerificationResult};
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command as ProcessCommand,
    str::FromStr,
};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
struct InternalProofRequest {
    public_key: [u8; 32],
    requested_amount: String,
    params: Vec<InternalHeuristic>,
}

impl From<ProofRequest> for InternalProofRequest {
    fn from(value: ProofRequest) -> Self {
        log::debug!("Params: {:?}", value);
        let pk: PublicKey = PublicKey::from_str(&value.public_key).expect("Invalid public key");
        let mut params = vec![InternalHeuristic::default(); 2];

        let mut internal_heuristics: Vec<InternalHeuristic> = value
            .params
            .into_iter()
            .map(InternalHeuristic::from)
            .collect();
        internal_heuristics.sort_by(|x, y| x.id.cmp(&y.id));

        for param in internal_heuristics {
            let id = param.id - 1;
            params[id as usize] = param;
        }

        InternalProofRequest {
            public_key: pk.key_data().try_into().unwrap(),
            requested_amount: value.requested_amount.to_string(),
            params,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct InternalHeuristic {
    id: u8,
    params: [String; 16],
}

impl InternalHeuristic {
    fn sparse_params() -> [String; 16] {
        let default_param = "0".to_string();
        let mut params = vec![];
        params.resize(16, default_param);
        assert_eq!(params.len(), 16);
        params.try_into().unwrap()
    }
}

impl Default for InternalHeuristic {
    fn default() -> Self {
        InternalHeuristic {
            id: 0,
            params: Self::sparse_params(),
        }
    }
}

impl From<Heuristic> for InternalHeuristic {
    fn from(value: Heuristic) -> Self {
        let mut params = Self::sparse_params();

        match value {
            Heuristic::Simple { balance } => params[0] = balance.to_string(),
        }

        InternalHeuristic {
            id: value.into(),
            params: params.try_into().unwrap(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct InternalVerificationRequest {
    public_key: [u8; 32],
    requested_amount: String,
}

impl From<Proof> for InternalVerificationRequest {
    fn from(value: Proof) -> Self {
        let pk: PublicKey = PublicKey::from_str(&value.public_key).expect("Invalid public key");
        InternalVerificationRequest {
            public_key: pk.key_data().try_into().unwrap(),
            requested_amount: value.requested_amount.to_string(),
        }
    }
}

pub enum Command {
    Prove(ProofRequest),
    Verify(Proof),
}

fn bootstrap_command<'process>(
    command: &Command,
    base_command: &'process mut ProcessCommand,
    path: &Path,
) -> &'process mut ProcessCommand {
    let action = match command {
        Command::Prove(_) => "prove",
        Command::Verify(_) => "verify",
    };
    let file_arg = match command {
        Command::Prove(_) => "--prover-name",
        Command::Verify(_) => "--verifier-name",
    };
    base_command
        .arg(action)
        .arg("--package")
        .arg("apply")
        .arg(file_arg)
        .arg(path)
}

pub fn execute<'a, T: TryFrom<(&'a Config, String, Command), Error = Report>>(
    config: &'a Config,
    command: Command,
) -> Result<T> {
    // This is to be sure that tempdir doesn't get forcibly dropped
    let temp_dir = tempfile::tempdir()?;
    execute_inner(config, command, &temp_dir)
}

/// Execute a call to nargo.
///
/// This process approach is a bit of a hack for the purpose of speed. Ideally
/// we would just integrate with the barretenberg backend directly.
pub fn execute_inner<'a, T: TryFrom<(&'a Config, String, Command), Error = Report>>(
    config: &'a Config,
    command: Command,
    temp_dir: &TempDir,
) -> Result<T> {
    let path = temp_dir.path().join("Params.toml");
    let mut file = File::create(&path)?;
    log::debug!("Created file: {:?}", path);

    let mut process = ProcessCommand::new("nargo");
    let process = bootstrap_command(&command, &mut process, &path);
    log::debug!("Built process {:?}", process);

    // Check if we need to flush or not
    match &command {
        Command::Prove(req) => {
            let internal = InternalProofRequest::from(req.clone());
            let toml_str = toml::to_string_pretty(&internal)?;
            log::debug!("Proof TOML: {}", toml_str);
            file.write_all(toml_str.as_bytes())?;
        }
        Command::Verify(req) => {
            let internal = InternalVerificationRequest::from(req.clone());
            let toml_str = toml::to_string_pretty(&internal)?;
            log::debug!("Verify TOML: {}", toml_str);
            file.write_all(toml_str.as_bytes())?;

            let mut proof_file = proof_file(config, false)?;
            proof_file.write_all(&hex::encode(&req.inner).as_bytes())?;
        }
    }

    log::debug!("Executing {:?}", process);
    let result = process
        .current_dir(&config.nargo_workspace_dir)
        .spawn()?
        .wait_with_output()?;

    log::debug!("Command result: {:?}", result);

    if result.status.success() {
        let stdout = String::from_utf8_lossy(&result.stdout);
        log::info!("Output {}", stdout);
        (config, stdout.to_string(), command).try_into()
    } else {
        let stderr = String::from_utf8_lossy(&result.stderr);
        log::error!("{}", stderr);
        Err(eyre::eyre!("nargo failed, stderr: {}", stderr))
    }
}

impl<'a> TryFrom<(&'a Config, String, Command)> for Proof {
    type Error = Report;

    fn try_from(value: (&Config, String, Command)) -> Result<Self> {
        let (config, _, cmd) = value;

        let mut proof_file = proof_file(config, true)?;

        let mut proof_hex = String::new();
        proof_file.read_to_string(&mut proof_hex)?;

        let (public_key, requested_amount) = match cmd {
            Command::Prove(r) => (r.public_key, r.requested_amount),
            Command::Verify(r) => (r.public_key, r.requested_amount),
        };

        Ok(Proof {
            requested_amount,
            public_key,
            inner: hex::decode(proof_hex)?,
            account_id: None,
        })
    }
}

impl<'a> TryFrom<(&'a Config, String, Command)> for VerificationResult {
    type Error = Report;
    fn try_from(_: (&Config, String, Command)) -> std::result::Result<Self, Self::Error> {
        Ok(VerificationResult(true))
    }
}

fn proof_dir(config: &Config) -> PathBuf {
    config
        .nargo_workspace_dir
        .join("proofs")
        .join("apply.proof")
}

fn proof_file(config: &Config, read_only: bool) -> Result<File> {
    let proof_path = proof_dir(config);

    #[cfg(test)]
    let proof_path = PathBuf::from("../../proofs/apply.proof");
    let file = if read_only {
        File::open(proof_path)
    } else {
        File::create(proof_path)
    }?;
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_crypto::{SecretKey, Signature};
    use std::str::FromStr;

    fn get_config() -> crate::config::Config {
        crate::config::Config::from("../../config")
    }

    #[test]
    fn proof_dir_from_config() {
        let config = get_config();
        let dir = proof_dir(&config);
        assert_eq!(
            dir,
            PathBuf::from("/home/common/projects/GhostFi/proofs/apply.proof")
        );
    }

    #[test]
    fn test_sparse_params() {
        let h = InternalHeuristic::default();
        assert_eq!(h.id, 0);
        let params = h.params;
        assert_eq!(params.len(), 16);
        for param in params {
            assert_eq!(param, "0");
        }
    }

    #[test]
    fn test_bootstrap_command() {
        let json = std::fs::read_to_string("fixtures/simple.json").unwrap();
        let proof: ProofRequest = serde_json::from_str(&json).unwrap();
        //let cmd = bootstrap_command(&Command::Prove(proof), Path::new("/tmp/hello.toml"));
        // assert_eq!(
        //     cmd,
        //     "nargo prove --package apply --prover-name /tmp/hello.toml"
        // );
    }

    #[test]
    fn test_prove_e2e() {
        let _ = pretty_env_logger::try_init();
        let config = get_config();
        println!("{:?}", config);
        let json = std::fs::read_to_string("fixtures/simple.json").unwrap();
        let proof: ProofRequest = serde_json::from_str(&json).unwrap();
        let proof: Proof = execute(&config, Command::Prove(proof)).unwrap();
        println!("{:?}", serde_json::to_string_pretty(&proof).unwrap());
    }

    #[test]
    fn test_verify_e2e() {
        let _ = pretty_env_logger::try_init();
        let config = get_config();
        println!("{:?}", config);
        let json = std::fs::read_to_string("fixtures/proof.json").unwrap();
        let proof: Proof = serde_json::from_str(&json).unwrap();
        let proven: VerificationResult = execute(&config, Command::Verify(proof)).unwrap();
        println!("Verified: {:?}", proven.0);
    }

    #[test]
    fn test_convert_to_inner() {
        let json = std::fs::read_to_string("fixtures/simple.json").unwrap();
        let proof: ProofRequest = serde_json::from_str(&json).unwrap();
        let inner = InternalProofRequest::from(proof);
        let toml = toml::to_string_pretty(&inner).unwrap();
        println!("{}", toml);
        let toml = std::fs::read_to_string("../../circuits/apply/Prover.toml").unwrap();
        let deser_inner = toml::from_str::<InternalProofRequest>(&toml).unwrap();
        assert_eq!(inner.public_key, deser_inner.public_key);
        assert_eq!(inner.requested_amount, deser_inner.requested_amount);
        assert_eq!(inner.params[0], deser_inner.params[0]);
    }

    #[test]
    fn sample_secp256k1_insane_noir_craziness() {
        //let signer = InMemorySigner::
        let privkey = near_crypto::SecretKey::from_random(near_crypto::KeyType::SECP256K1);
        let privkey = near_crypto::SecretKey::from_str(
            "secp256k1:TFbJKaj5cjQ5fhirH75cNqhuMEUFfZZ7TSyaVCAYGzb",
        )
        .unwrap();
        println!("{}", privkey);
        if let SecretKey::SECP256K1(inner) = privkey {
            println!("Pkey: {:?}", inner.secret_bytes());
            println!("Pkey: {:?}", hex::encode(inner.secret_bytes()));
        }

        let pubkey = privkey.public_key();
        let pubkey = pubkey.key_data();
        println!("X: {:?}", &pubkey[0..32]);
        println!("Y: {:?}", &pubkey[32..64]);

        let msg = [1u8; 32];

        let signature = privkey.sign(&msg);

        if let Signature::SECP256K1(inner) = signature {
            let bytes: [u8; 65] = inner.clone().into();
            println!("Sig: {:?}", &bytes[0..64]);
            println!("Sig X: {:?}", &bytes[0..32]);
            println!("Sig Y: {:?}", &bytes[32..64]);

            let verified = acvm::blackbox_solver::ecdsa_secp256k1_verify(
                &msg,
                pubkey[0..32].try_into().unwrap(),
                pubkey[32..64].try_into().unwrap(),
                bytes[0..64].try_into().unwrap(),
            )
            .unwrap();
            assert!(verified);

            let sig_x1 = &bytes[0..16];
            let sig_x2 = &bytes[16..32];
            let sig_y1 = &bytes[32..48];
            let sig_y2 = &bytes[48..64];

            let concat: [u8; 64] = vec![
                sig_x1.to_vec(),
                sig_x2.to_vec(),
                sig_y1.to_vec(),
                sig_y2.to_vec(),
            ]
            .concat()
            .try_into()
            .unwrap();
            println!("Concat: {:?}", concat);
            assert_eq!(bytes[0..64], concat);

            let verified = acvm::blackbox_solver::ecdsa_secp256k1_verify(
                &msg,
                pubkey[0..32].try_into().unwrap(),
                pubkey[32..64].try_into().unwrap(),
                &concat,
            )
            .unwrap();
            assert!(verified);

            let sig_x1 = hex::encode(&sig_x1);
            let sig_x2 = hex::encode(&sig_x2);
            let sig_y1 = hex::encode(&sig_y1);
            let sig_y2 = hex::encode(&sig_y2);
            let msg = hex::encode(&msg);
            println!(
                "
                \"0x{}\", 
                \"0x{}\",
                \"0x{}\",
                \"0x{}\"
                \"0x{}\"
                ",
                sig_x1, sig_x2, sig_y1, sig_y2, msg
            )
        }
    }
}
