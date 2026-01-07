use actix_web::{HttpRequest};

/// Tries to get a single header as String
pub fn extract_header(req: &HttpRequest, name: &str) -> Option<String> {
    req.headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Best-effort IP extraction.
///
/// IMPORTANT:
/// - If you are behind a reverse proxy (Railway / Cloudflare / Nginx),
///   prefer trusted headers (CF-Connecting-IP, X-Real-IP, X-Forwarded-For).
/// - Never blindly trust X-Forwarded-For if you are not behind a trusted proxy.
pub fn extract_client_ip(req: &HttpRequest) -> Option<String> {
    // Cloudflare
    if let Some(ip) = extract_header(req, "cf-connecting-ip") {
        return Some(ip);
    }

    // Nginx / proxy
    if let Some(ip) = extract_header(req, "x-real-ip") {
        return Some(ip);
    }

    // X-Forwarded-For: "client, proxy1, proxy2"
    if let Some(xff) = extract_header(req, "x-forwarded-for") {
        if let Some(first) = xff.split(',').next().map(|s| s.trim()).filter(|s| !s.is_empty()) {
            return Some(first.to_string());
        }
    }

    // Actix connection info (may be proxy-aware depending on config)
    req.connection_info()
        .realip_remote_addr()
        .map(|s| s.to_string())
}
