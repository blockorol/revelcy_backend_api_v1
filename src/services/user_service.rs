use actix_web::error::ErrorInternalServerError;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config;
use crate::models::user::{ApplyInviteCodeResult, User, UserShort};
use crate::services::file_service;
use crate::storage::user_repo;

const GET_IMAGE_PATH: &str = "files/image/";

pub async fn set_invite_code_once(
    pool: &PgPool,
    user_id: Uuid,
    invite_code: &str,
) -> Result<ApplyInviteCodeResult, actix_web::Error> {
    user_repo::apply_invite_code_once(pool, user_id, invite_code)
        .await
        .map_err(ErrorInternalServerError)
}

pub async fn set_username(
    pool: &PgPool,
    user_id: Uuid,
    user_name: &str,
) -> Result<(), actix_web::Error> {
    user_repo::update_username(pool, user_id, user_name)
        .await
        .map_err(ErrorInternalServerError)
}

pub async fn set_avatar(
    pool: &PgPool,
    user_id: Uuid,
    avatar: &[u8],
) -> Result<String, actix_web::Error> {
    let path = file_service::save_png(&user_id.to_string(), avatar)
        .map_err(ErrorInternalServerError)?;

    let host = config::get_host();
    let url = format!("{host}{GET_IMAGE_PATH}{path}");

    user_repo::update_avatar_url(pool, user_id, &url)
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(url)
}

/// get-or-create для wallet address
pub async fn get_or_create_by_wallet_address(
    pool: &PgPool,
    wallet_address: &str,
) -> Result<User, actix_web::Error> {
    if let Some(user) = user_repo::get_user_by_wallet(pool, wallet_address)
        .await
        .map_err(ErrorInternalServerError)?
    {
        return Ok(user);
    }

    user_repo::create_user_with_wallet(pool, wallet_address)
        .await
        .map_err(ErrorInternalServerError)
}

/// strict get by wallet address (без создания)
pub async fn get_by_wallet_address(
    pool: &PgPool,
    wallet_address: &str,
) -> Result<Option<User>, actix_web::Error> {
    user_repo::get_user_by_wallet(pool, wallet_address)
        .await
        .map_err(ErrorInternalServerError)
}

pub async fn search_by_username(
    pool: &PgPool,
    input: &str,
    limit: i64,
) -> Result<Vec<User>, actix_web::Error> {
    user_repo::search_users_by_username_with_wallets(pool, input, limit)
        .await
        .map_err(ErrorInternalServerError)
}

pub async fn get_users_short_by_addresses(
    pool: &PgPool,
    addresses: &[String],
) -> Result<Vec<UserShort>, actix_web::Error> {
    user_repo::get_users_short_by_addresses(pool, addresses)
        .await
        .map_err(ErrorInternalServerError)
}
