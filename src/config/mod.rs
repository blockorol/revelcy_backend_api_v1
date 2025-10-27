pub fn get_host() -> String {
    std::env::var("CURRENT_HOST").unwrap_or_else(|_| "http://localhost:8080/".to_string())
}
pub fn get_jwt_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "SECRET_super_puper".to_string())
}
pub fn get_pyth_subdomain() -> String {
    std::env::var("PYTH_SUBDOMAIN").unwrap_or_else(|_| "api".to_string())
}
pub fn get_pyth_secret_token() -> String {
    std::env::var("PYTH_SECRET_TOKEN").unwrap_or_else(|_| "".to_string())
}