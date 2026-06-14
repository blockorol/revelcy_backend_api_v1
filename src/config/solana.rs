use super::env::{get_optional_env, get_required_env};
use std::env::VarError;

pub const SOLANA_RPC_ENV: &str = "SOLANA_RPC";
pub const SOLANA_DEVNET_RPC_ENV: &str = "SOLANA_DEVNET_RPC";
pub const SOLANA_MAINNET_RPC_ENV: &str = "SOLANA_MAINNET_RPC";
pub const NETWORK_ENV: &str = "NETWORK";
pub const PURPLE_PROGRAM_ID_DEV_ENV: &str = "PURPLE_PROGRAM_ID_DEV";
pub const PURPLE_PROGRAM_ID_MAIN_ENV: &str = "PURPLE_PROGRAM_ID_MAIN";

pub fn get_solana_rpc() -> Result<String, VarError> {
    get_required_env(SOLANA_RPC_ENV)
}

pub fn get_solana_devnet_rpc() -> String {
    std::env::var(SOLANA_DEVNET_RPC_ENV)
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string())
}

pub fn get_solana_mainnet_rpc() -> String {
    std::env::var(SOLANA_MAINNET_RPC_ENV)
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

pub fn get_network() -> String {
    std::env::var(NETWORK_ENV).unwrap_or_else(|_| "".to_string())
}

pub fn get_purple_program_id_dev() -> Option<String> {
    get_optional_env(PURPLE_PROGRAM_ID_DEV_ENV)
}

pub fn get_purple_program_id_main() -> Option<String> {
    get_optional_env(PURPLE_PROGRAM_ID_MAIN_ENV)
}
