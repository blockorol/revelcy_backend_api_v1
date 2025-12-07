use actix_cors::Cors;
use actix_web::http;
use std::env;

pub fn cors_middleware() -> Cors {
    let mut cors = Cors::default()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allowed_headers(vec![
            http::header::AUTHORIZATION,
            http::header::CONTENT_TYPE,
        ])
        .max_age(3600);

    let origins = env::var("CORS_ORIGINS").unwrap_or_default();
    if origins.trim().is_empty() {
        eprintln!("⚠️  CORS_ORIGINS not set — allow_any_origin()");
        cors = cors.allow_any_origin();
    } else {
        for origin in origins.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            println!("🔐 Allow CORS origin: {}", origin);
            cors = cors.allowed_origin(origin);
        }
    }

    cors
}
