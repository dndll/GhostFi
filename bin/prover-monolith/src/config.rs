use config::{Config as ConfigExt, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub account: String,
    pub secret: String,
    pub proof_path: String,
    pub rpc: String,
    pub circuit_workspace: String,
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
