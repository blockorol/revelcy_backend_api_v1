use std::error::Error;
use std::fmt;

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::whitelist::{WhitelistStatus, WhitelistUsersResult};
use crate::services::user_service;
use crate::storage::whitelist_repo;

#[derive(Debug)]
pub enum WhitelistServiceError {
    MissingUserIdentifier,
    UserLookup(String),
    Storage(sqlx::Error),
}

impl fmt::Display for WhitelistServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingUserIdentifier => write!(f, "user_id or user_wallet_address required"),
            Self::UserLookup(err) => write!(f, "user lookup failed: {err}"),
            Self::Storage(err) => write!(f, "storage error: {err}"),
        }
    }
}

impl Error for WhitelistServiceError {}

impl From<sqlx::Error> for WhitelistServiceError {
    fn from(value: sqlx::Error) -> Self {
        Self::Storage(value)
    }
}

pub type WhitelistServiceResult<T> = Result<T, WhitelistServiceError>;

pub async fn is_user_whitelisted(
    pool: &PgPool,
    premarket_id: Uuid,
    user_id: Uuid,
) -> WhitelistServiceResult<bool> {
    whitelist_repo::exists(pool, premarket_id, user_id)
        .await
        .map_err(Into::into)
}

/// 2) Добавить 1 пользователя в whitelist:
/// - либо по user_id
/// - либо по wallet_address (public key), с автосозданием пользователя
pub async fn add_user(
    pool: &PgPool,
    premarket_id: Uuid,
    user_id: Option<Uuid>,
    user_wallet_address: Option<&str>,
) -> WhitelistServiceResult<()> {
    let uid = match (user_id, user_wallet_address) {
        (Some(id), _) => id,
        (None, Some(addr)) => {
            let user = user_service::get_or_create_by_wallet_address(pool, addr)
                .await
                .map_err(|e| WhitelistServiceError::UserLookup(e.to_string()))?;
            user.id
        }
        _ => return Err(WhitelistServiceError::MissingUserIdentifier),
    };

    whitelist_repo::add(pool, premarket_id, uid, WhitelistStatus::Approved)
        .await
        .map_err(WhitelistServiceError::Storage)?;

    Ok(())
}

/// 3) Добавить набор пользователей в whitelist:
/// - либо по user_ids
/// - либо по wallet_addresses
/// - можно оба (объединит)
pub async fn add_users(
    pool: &PgPool,
    premarket_id: Uuid,
    user_ids: Option<Vec<Uuid>>,
    user_wallet_addresses: Option<Vec<String>>,
) -> WhitelistServiceResult<()> {
    let mut ids: Vec<Uuid> = user_ids.unwrap_or_default();

    if let Some(addrs) = user_wallet_addresses {
        for addr in addrs {
            let user = user_service::get_or_create_by_wallet_address(pool, &addr)
                .await
                .map_err(|e| WhitelistServiceError::UserLookup(e.to_string()))?;
            ids.push(user.id);
        }
    }

    if ids.is_empty() {
        return Ok(());
    }

    ids.sort();
    ids.dedup();

    whitelist_repo::add_many(pool, premarket_id, &ids, WhitelistStatus::Approved)
        .await
        .map_err(WhitelistServiceError::Storage)?;

    Ok(())
}

pub async fn apply_user(
    pool: &PgPool,
    premarket_id: Uuid,
    user_id: Uuid,
) -> WhitelistServiceResult<bool> {
    whitelist_repo::add_requested_if_absent(pool, premarket_id, user_id)
        .await
        .map_err(Into::into)
}

pub async fn get_users(
    pool: &PgPool,
    premarket_id: Uuid,
    status: Option<WhitelistStatus>,
    cursor: i64,
    limit: i64,
) -> WhitelistServiceResult<WhitelistUsersResult> {
    let res = match status {
        None => whitelist_repo::list_users_by_premarket(pool, premarket_id, cursor, limit).await,
        Some(s) => whitelist_repo::list_users_by_status(pool, premarket_id, s, cursor, limit).await,
    };
    let (items, total) = res.map_err(WhitelistServiceError::Storage)?;

    Ok(WhitelistUsersResult {
        items,
        total: Some(total),
    })
}

pub async fn remove_user(
    pool: &PgPool,
    premarket_id: Uuid,
    user_id: Option<Uuid>,
    user_wallet_address: Option<&str>,
) -> WhitelistServiceResult<u64> {
    let uid = match (user_id, user_wallet_address) {
        (Some(id), _) => id,
        (None, Some(addr)) => {
            let user_opt = user_service::get_by_wallet_address(pool, addr)
                .await
                .map_err(|e| WhitelistServiceError::UserLookup(e.to_string()))?;

            match user_opt {
                Some(u) => u.id,
                None => return Ok(0),
            }
        }
        _ => return Err(WhitelistServiceError::MissingUserIdentifier),
    };

    let affected = whitelist_repo::delete_by_premarket_user(pool, premarket_id, uid)
        .await
        .map_err(WhitelistServiceError::Storage)?;

    Ok(affected)
}

pub async fn approve_user(
    pool: &PgPool,
    premarket_id: Uuid,
    user_id: Uuid,
) -> WhitelistServiceResult<()> {
    set_user_status(pool, premarket_id, user_id, WhitelistStatus::Approved).await
}

pub async fn reject_user(
    pool: &PgPool,
    premarket_id: Uuid,
    user_id: Uuid,
) -> WhitelistServiceResult<()> {
    set_user_status(pool, premarket_id, user_id, WhitelistStatus::Rejected).await
}

pub async fn set_user_status(
    pool: &PgPool,
    premarket_id: Uuid,
    user_id: Uuid,
    status: WhitelistStatus,
) -> WhitelistServiceResult<()> {
    whitelist_repo::update_status(pool, premarket_id, user_id, status)
        .await
        .map_err(Into::into)
}
