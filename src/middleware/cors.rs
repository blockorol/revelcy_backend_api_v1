use crate::config;
use actix_cors::Cors;
// use actix_web::http;

pub fn cors_middleware() -> Cors {
    let mut cors = Cors::default()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allow_any_header()
        // .allowed_headers(vec![
        //     http::header::AUTHORIZATION,
        //     http::header::CONTENT_TYPE,
        // ])
        .max_age(3600);

    let origins = config::get_cors_origins();
    if origins.trim().is_empty() {
        eprintln!(
            "⚠️  {} not set — allow_any_origin()",
            config::CORS_ORIGINS_ENV
        );
        cors = cors.allow_any_origin();
    } else {
        println!("🔐 Allow CORS with settings: {}", origins);
        for origin in origins
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            println!("🔐 Allow CORS origin: {}", origin);
            cors = cors.allowed_origin(origin);
        }
    }

    cors
}
