use super::env::get_required_env;
use std::env::VarError;

pub const DATABASE_URL_ENV: &str = "DATABASE_URL";

pub fn get_database_url() -> Result<String, VarError> {
    get_required_env(DATABASE_URL_ENV)
}
