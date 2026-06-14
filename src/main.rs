use crate::middleware::cors::cors_middleware;
use crate::server::init_servers;
use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use solana_client::nonblocking::rpc_client::RpcClient;
use sqlx::postgres::PgPoolOptions;

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

    // DB Connect
    let database_url = config::get_database_url()
        .unwrap_or_else(|_| panic!("{} must be set", config::DATABASE_URL_ENV));
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    let rpc_url = config::get_solana_rpc()
        .unwrap_or_else(|_| panic!("{} must be set", config::SOLANA_RPC_ENV));
    let rpc_client = web::Data::new(RpcClient::new(rpc_url));

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
