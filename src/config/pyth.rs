pub const PYTH_SUBDOMAIN_ENV: &str = "PYTH_SUBDOMAIN";
pub const PYTH_SECRET_TOKEN_ENV: &str = "PYTH_SECRET_TOKEN";
pub const PYTH_MAINNET_URL_ENV: &str = "PYTH_MAINNET_URL";

pub fn get_pyth_subdomain() -> String {
    std::env::var(PYTH_SUBDOMAIN_ENV).unwrap_or_else(|_| "api".to_string())
}

pub fn get_pyth_secret_token() -> String {
    std::env::var(PYTH_SECRET_TOKEN_ENV).unwrap_or_else(|_| "".to_string())
}

pub fn get_pyth_mainnet_url() -> String {
    std::env::var(PYTH_MAINNET_URL_ENV).unwrap_or_else(|_| "".to_string())
}
