use std::env::VarError;

pub fn get_required_env(name: &str) -> Result<String, VarError> {
    std::env::var(name)
}

pub fn get_optional_env(name: &str) -> Option<String> {
    std::env::var(name).ok()
}
