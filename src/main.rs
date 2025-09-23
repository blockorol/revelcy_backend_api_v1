use actix_web::{App, HttpServer, web};
use solana_client::nonblocking::rpc_client::RpcClient;
use crate::server::init_servers;
use crate::middleware::cors::cors_middleware;
use sqlx::postgres::PgPoolOptions;
use dotenvy::dotenv;
use std::env;

mod middleware;
mod server;
mod services;
mod api;
mod config;
mod models;
mod storage;
mod constants;

#[actix_web::main] 
async fn main() -> std::io::Result<()> {
    // load .env
    dotenv().ok();

    // DB Connect
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    let rpc_url = env::var("SOLANA_DEVNET_RPC").expect("SOLANA_DEVNET_RPC must be set");
    let rpc_client = web::Data::new(RpcClient::new(rpc_url)); 

    // run server
    HttpServer::new(move || {
        App::new()
            .wrap(cors_middleware())
            .app_data(web::Data::new(pool.clone()))
            .app_data(rpc_client.clone()) 
            .configure(init_servers)
    })
    .bind(format!("0.0.0.0:{}", std::env::var("PORT").unwrap_or_else(|_| "8080".to_string())))?
    .run()
    .await
}
