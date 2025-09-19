use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use uuid::Uuid;

use revelcy_backend_api::storage::user_repo;
use revelcy_backend_api::models::user::User;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(())
}
