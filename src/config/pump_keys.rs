pub const TARGET_SUFFIX_ENV: &str = "TARGET_SUFFIX";
pub const DEFAULT_TARGET_SUFFIX: &str = "pump";

pub fn get_target_suffix() -> String {
    std::env::var(TARGET_SUFFIX_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_TARGET_SUFFIX.to_string())
}
