use super::env::get_required_env;
use std::env::VarError;

pub const JWT_SECRET_ENV: &str = "JWT_SECRET";
pub const REVELCY_AUTH_PRIVATE_KEY_ENV: &str = "REVELCY_AUTH_PRIVATE_KEY";
pub const REVELCY_AUTH_PRIVATE_KEY_DEV_ENV: &str = "REVELCY_AUTH_PRIVATE_KEY_DEV";
pub const REVELCY_AUTH_PRIVATE_KEY_MAIN_ENV: &str = "REVELCY_AUTH_PRIVATE_KEY_MAIN";

pub fn get_jwt_secret() -> String {
    std::env::var(JWT_SECRET_ENV).unwrap_or_else(|_| "SECRET_super_puper".to_string())
}

pub fn get_revelcy_auth_privite_key() -> String {
    std::env::var(REVELCY_AUTH_PRIVATE_KEY_ENV).unwrap_or_else(|_| "".to_string())
}

pub fn get_revelcy_auth_private_key_dev() -> Result<String, VarError> {
    get_required_env(REVELCY_AUTH_PRIVATE_KEY_DEV_ENV)
}

pub fn get_revelcy_auth_private_key_main() -> Result<String, VarError> {
    get_required_env(REVELCY_AUTH_PRIVATE_KEY_MAIN_ENV)
}
