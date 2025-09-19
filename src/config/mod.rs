pub fn get_host() -> String {
    std::env::var("CURRENT_HOST").unwrap_or_else(|_| "http://localhost:8080/".to_string())
}
pub fn get_jwt_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "SECRET_super_puper".to_string())
}
