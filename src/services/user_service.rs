use std::error::Error;
use std::fmt;
use std::io;

use sqlx::PgPool;
use uuid::Uuid;

use crate::config;
use crate::models::user::{ApplyInviteCodeResult, User, UserShort};
use crate::services::file_service;
use crate::storage::user_repo;

const GET_IMAGE_PATH: &str = "files/image/";

#[derive(Debug)]
pub enum UserServiceError {
    Storage(sqlx::Error),
    File(io::Error),
}

impl fmt::Display for UserServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(err) => write!(f, "storage error: {err}"),
            Self::File(err) => write!(f, "file error: {err}"),
        }
    }
}

impl Error for UserServiceError {}

impl From<sqlx::Error> for UserServiceError {
    fn from(value: sqlx::Error) -> Self {
        Self::Storage(value)
    }
}

impl From<io::Error> for UserServiceError {
    fn from(value: io::Error) -> Self {
        Self::File(value)
    }
}

pub type UserServiceResult<T> = Result<T, UserServiceError>;

pub async fn set_invite_code_once(
    pool: &PgPool,
    user_id: Uuid,
    invite_code: &str,
) -> UserServiceResult<ApplyInviteCodeResult> {
    user_repo::apply_invite_code_once(pool, user_id, invite_code)
        .await
        .map_err(Into::into)
}

pub async fn set_username(pool: &PgPool, user_id: Uuid, user_name: &str) -> UserServiceResult<()> {
    user_repo::update_username(pool, user_id, user_name)
        .await
        .map_err(Into::into)
}

pub async fn set_avatar(pool: &PgPool, user_id: Uuid, avatar: &[u8]) -> UserServiceResult<String> {
    let path = file_service::save_png(&user_id.to_string(), avatar)?;

    let host = config::get_host();
    let url = format!("{host}{GET_IMAGE_PATH}{path}");

    user_repo::update_avatar_url(pool, user_id, &url)
        .await
        .map_err(UserServiceError::Storage)?;

    Ok(url)
}

/// get-or-create для wallet address
pub async fn get_or_create_by_wallet_address(
    pool: &PgPool,
    wallet_address: &str,
) -> UserServiceResult<User> {
    if let Some(user) = user_repo::get_user_by_wallet(pool, wallet_address)
        .await
        .map_err(UserServiceError::Storage)?
    {
        return Ok(user);
    }

    user_repo::create_user_with_wallet(pool, wallet_address)
        .await
        .map_err(Into::into)
}

/// strict get by wallet address (без создания)
pub async fn get_by_wallet_address(
    pool: &PgPool,
    wallet_address: &str,
) -> UserServiceResult<Option<User>> {
    user_repo::get_user_by_wallet(pool, wallet_address)
        .await
        .map_err(Into::into)
}

pub async fn search_by_username(
    pool: &PgPool,
    input: &str,
    limit: i64,
) -> UserServiceResult<Vec<User>> {
    user_repo::search_users_by_username_with_wallets(pool, input, limit)
        .await
        .map_err(Into::into)
}

pub async fn get_users_short_by_addresses(
    pool: &PgPool,
    addresses: &[String],
) -> UserServiceResult<Vec<UserShort>> {
    user_repo::get_users_short_by_addresses(pool, addresses)
        .await
        .map_err(Into::into)
}
