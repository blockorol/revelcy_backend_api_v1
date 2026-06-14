pub const PORT_ENV: &str = "PORT";
pub const CURRENT_HOST_ENV: &str = "CURRENT_HOST";
pub const CORS_ORIGINS_ENV: &str = "CORS_ORIGINS";

pub fn get_port() -> String {
    std::env::var(PORT_ENV).unwrap_or_else(|_| "8080".to_string())
}

pub fn get_host() -> String {
    std::env::var(CURRENT_HOST_ENV).unwrap_or_else(|_| "http://localhost:8080/".to_string())
}

pub fn get_cors_origins() -> String {
    std::env::var(CORS_ORIGINS_ENV).unwrap_or_default()
}
