use actix_cors::Cors;
use actix_web::http;
use std::env;

pub fn cors_middleware() -> Cors {
    let origins = env::var("CORS_ORIGINS").unwrap_or_default();

    let mut cors = 
    Cors::default()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allowed_headers(vec![
            http::header::AUTHORIZATION,
            http::header::CONTENT_TYPE,
        ])
        // .supports_credentials()
        .max_age(3600);

    for origin in origins.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
        cors = cors.allowed_origin(origin);
    }

    cors
}
