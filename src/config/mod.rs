use std::fmt;

mod env;

pub mod database;
pub mod pump_keys;
pub mod pyth;
pub mod security;
pub mod server;
pub mod solana;
pub mod storage;

pub use database::*;
pub use env::{get_optional_env, get_required_env};
pub use pump_keys::*;
pub use pyth::*;
pub use security::*;
pub use server::*;
pub use solana::*;
pub use storage::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError {
    missing_required: Vec<&'static str>,
}

impl ConfigError {
    fn missing(required: Vec<&'static str>) -> Self {
        Self {
            missing_required: required,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.missing_required.is_empty()
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "missing required environment variables: {}",
            self.missing_required.join(", ")
        )
    }
}

impl std::error::Error for ConfigError {}

pub fn validate_startup_config() -> Result<(), ConfigError> {
    validate_required_envs(&[
        DATABASE_URL_ENV,
        SOLANA_RPC_ENV,
        JWT_SECRET_ENV,
        CURRENT_HOST_ENV,
        PYTH_MAINNET_URL_ENV,
        REVELCY_AUTH_PRIVATE_KEY_DEV_ENV,
        REVELCY_AUTH_PRIVATE_KEY_MAIN_ENV,
    ])
}

pub fn validate_pump_keys_config() -> Result<(), ConfigError> {
    validate_required_envs(&[DATABASE_URL_ENV])
}

fn validate_required_envs(required: &[&'static str]) -> Result<(), ConfigError> {
    let missing_required = required
        .into_iter()
        .filter(|name| {
            get_optional_env(name)
                .map(|value| value.trim().is_empty())
                .unwrap_or(true)
        })
        .copied()
        .collect::<Vec<_>>();

    if missing_required.is_empty() {
        Ok(())
    } else {
        Err(ConfigError::missing(missing_required))
    }
}
