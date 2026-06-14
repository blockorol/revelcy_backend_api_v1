pub const STORAGE_DIR_ENV: &str = "STORAGE_DIR";

pub fn get_storage_dir() -> String {
    std::env::var(STORAGE_DIR_ENV).unwrap_or_else(|_| "./storage".to_string())
}
