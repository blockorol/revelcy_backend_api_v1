use std::env::VarError;

pub const DATABASE_URL_ENV: &str = "DATABASE_URL";
pub const SOLANA_RPC_ENV: &str = "SOLANA_RPC";
pub const SOLANA_DEVNET_RPC_ENV: &str = "SOLANA_DEVNET_RPC";
pub const SOLANA_MAINNET_RPC_ENV: &str = "SOLANA_MAINNET_RPC";
pub const PORT_ENV: &str = "PORT";
pub const CURRENT_HOST_ENV: &str = "CURRENT_HOST";
pub const JWT_SECRET_ENV: &str = "JWT_SECRET";
pub const PYTH_SUBDOMAIN_ENV: &str = "PYTH_SUBDOMAIN";
pub const PYTH_SECRET_TOKEN_ENV: &str = "PYTH_SECRET_TOKEN";
pub const REVELCY_AUTH_PRIVATE_KEY_ENV: &str = "REVELCY_AUTH_PRIVATE_KEY";
pub const REVELCY_AUTH_PRIVATE_KEY_DEV_ENV: &str = "REVELCY_AUTH_PRIVATE_KEY_DEV";
pub const REVELCY_AUTH_PRIVATE_KEY_MAIN_ENV: &str = "REVELCY_AUTH_PRIVATE_KEY_MAIN";
pub const PYTH_MAINNET_URL_ENV: &str = "PYTH_MAINNET_URL";
pub const STORAGE_DIR_ENV: &str = "STORAGE_DIR";
pub const CORS_ORIGINS_ENV: &str = "CORS_ORIGINS";
pub const TARGET_SUFFIX_ENV: &str = "TARGET_SUFFIX";
pub const NETWORK_ENV: &str = "NETWORK";
pub const PURPLE_PROGRAM_ID_DEV_ENV: &str = "PURPLE_PROGRAM_ID_DEV";
pub const PURPLE_PROGRAM_ID_MAIN_ENV: &str = "PURPLE_PROGRAM_ID_MAIN";

pub fn get_required_env(name: &str) -> Result<String, VarError> {
    std::env::var(name)
}

pub fn get_optional_env(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

pub fn get_database_url() -> Result<String, VarError> {
    get_required_env(DATABASE_URL_ENV)
}

pub fn get_solana_rpc() -> Result<String, VarError> {
    get_required_env(SOLANA_RPC_ENV)
}

pub fn get_revelcy_auth_private_key_dev() -> Result<String, VarError> {
    get_required_env(REVELCY_AUTH_PRIVATE_KEY_DEV_ENV)
}

pub fn get_revelcy_auth_private_key_main() -> Result<String, VarError> {
    get_required_env(REVELCY_AUTH_PRIVATE_KEY_MAIN_ENV)
}

pub fn get_purple_program_id_dev() -> Option<String> {
    get_optional_env(PURPLE_PROGRAM_ID_DEV_ENV)
}

pub fn get_purple_program_id_main() -> Option<String> {
    get_optional_env(PURPLE_PROGRAM_ID_MAIN_ENV)
}

pub fn get_port() -> String {
    std::env::var(PORT_ENV).unwrap_or_else(|_| "8080".to_string())
}

pub fn get_host() -> String {
    std::env::var(CURRENT_HOST_ENV).unwrap_or_else(|_| "http://localhost:8080/".to_string())
}
pub fn get_jwt_secret() -> String {
    std::env::var(JWT_SECRET_ENV).unwrap_or_else(|_| "SECRET_super_puper".to_string())
}
pub fn get_pyth_subdomain() -> String {
    std::env::var(PYTH_SUBDOMAIN_ENV).unwrap_or_else(|_| "api".to_string())
}
pub fn get_pyth_secret_token() -> String {
    std::env::var(PYTH_SECRET_TOKEN_ENV).unwrap_or_else(|_| "".to_string())
}
pub fn get_revelcy_auth_privite_key() -> String {
    std::env::var(REVELCY_AUTH_PRIVATE_KEY_ENV).unwrap_or_else(|_| "".to_string())
}
pub fn get_pyth_mainnet_url() -> String {
    std::env::var(PYTH_MAINNET_URL_ENV).unwrap_or_else(|_| "".to_string())
}
pub fn get_storage_dir() -> String {
    std::env::var(STORAGE_DIR_ENV).unwrap_or_else(|_| "./storage".to_string())
}
pub fn get_cors_origins() -> String {
    std::env::var(CORS_ORIGINS_ENV).unwrap_or_default()
}
pub fn get_solana_devnet_rpc() -> String {
    std::env::var(SOLANA_DEVNET_RPC_ENV)
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string())
}
pub fn get_solana_mainnet_rpc() -> String {
    std::env::var(SOLANA_MAINNET_RPC_ENV)
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}
pub fn get_target_suffix() -> Result<String, VarError> {
    get_required_env(TARGET_SUFFIX_ENV)
}
// todo: change to SolanaNetwork
pub fn get_network() -> String {
    std::env::var(NETWORK_ENV).unwrap_or_else(|_| "".to_string())
}

// todo: Network -> enum (mainnet/devnet)
// hosts -> Url
// add vaildation on startup
// make a config with banch
