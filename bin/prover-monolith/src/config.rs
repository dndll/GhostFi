use std::path::PathBuf;
use config::{Config as ConfigExt, ConfigError, Environment, File};
use near_account_id::AccountId;
use near_crypto::SecretKey;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// The near account for integration with the contract
    pub account: AccountId,
    /// The private key for the account that integrates with the contract
    pub secret: SecretKey,
    /// The near rpc provider
    pub rpc: String,
    /// The circuits location, usually where `Nargo.toml` is
    pub nargo_workspace_dir: PathBuf,
}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let s = ConfigExt::builder()
            .add_source(File::with_name("config"))
            // This file shouldn't be checked in to git
            .add_source(File::with_name("local").required(false))
            // Add in settings from the environment (with a prefix of APP)
            .add_source(Environment::with_prefix("GHOSTFI"))
            .build()?;

        s.try_deserialize()
    }
}

#[cfg(test)]
impl From<&str> for Config {
    fn from(s: &str) -> Self {
        let s = ConfigExt::builder()
            .add_source(File::with_name("config").required(false))
            // This file shouldn't be checked in to git
            .add_source(File::with_name(s))
            // Add in settings from the environment (with a prefix of APP)
            .add_source(Environment::with_prefix("GHOSTFI"))
            .build()
            .unwrap();

        s.try_deserialize().unwrap()
    }
}
