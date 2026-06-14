use crate::middleware::cors::cors_middleware;
use crate::server::init_servers;
use crate::services::solana_rpc_client;
use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

mod api;
mod config;
mod constants;
mod middleware;
mod models;
mod server;
mod services;
mod storage;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // load .env
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    config::validate_startup_config().unwrap_or_else(|err| panic!("{err}"));

    // DB Connect
    let database_url = config::get_database_url()
        .unwrap_or_else(|_| panic!("{} must be set", config::DATABASE_URL_ENV));
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    let rpc_client = web::Data::new(
        solana_rpc_client::make_default_async_rpc_client()
            .unwrap_or_else(|_| panic!("{} must be set", config::SOLANA_RPC_ENV)),
    );

    // run server
    HttpServer::new(move || {
        App::new()
            .wrap(cors_middleware())
            .app_data(web::Data::new(pool.clone()))
            .app_data(rpc_client.clone())
            .configure(init_servers)
    })
    .bind(format!("0.0.0.0:{}", config::get_port()))?
    .run()
    .await
}
