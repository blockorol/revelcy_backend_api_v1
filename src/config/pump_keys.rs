use super::env::get_required_env;
use std::env::VarError;

pub const TARGET_SUFFIX_ENV: &str = "TARGET_SUFFIX";

pub fn get_target_suffix() -> Result<String, VarError> {
    get_required_env(TARGET_SUFFIX_ENV)
}
