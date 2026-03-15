use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::whitelist::{WhitelistUsersResult, WhitelistStatus};
use crate::services::user_service;
use crate::storage::whitelist_repo;


pub async fn is_user_whitelisted(
    pool: &PgPool,
    premarket_id: Uuid,
    user_id: Uuid,
) -> Result<bool, actix_web::Error> {
    whitelist_repo::exists(pool, premarket_id, user_id)
        .await
        .map_err(ErrorInternalServerError)
}


/// 2) Добавить 1 пользователя в whitelist:
/// - либо по user_id
/// - либо по wallet_address (public key), с автосозданием пользователя
pub async fn add_user(
    pool: &PgPool,
    premarket_id: Uuid,
    user_id: Option<Uuid>,
    user_wallet_address: Option<&str>,
) -> Result<(), actix_web::Error> {
    let uid = match (user_id, user_wallet_address) {
        (Some(id), _) => id,
        (None, Some(addr)) => {
            let user = user_service::get_or_create_by_wallet_address(pool, addr).await?;
            user.id
        },
        _ => return Err(ErrorBadRequest("user_id or user_wallet_address required")),
    };

    whitelist_repo::add(pool, premarket_id, uid, WhitelistStatus::Approved)
        .await
        .map_err(ErrorInternalServerError)?;

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
) -> Result<(), actix_web::Error> {
    let mut ids: Vec<Uuid> = user_ids.unwrap_or_default();

    if let Some(addrs) = user_wallet_addresses {
        for addr in addrs {
            let user = user_service::get_or_create_by_wallet_address(pool, &addr).await?;
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
        .map_err(ErrorInternalServerError)?;

    Ok(())
}

pub async fn apply_user(
    pool: &PgPool,
    premarket_id: Uuid,
    user_id: Uuid,
) -> Result<bool, actix_web::Error> {
    whitelist_repo::add_requested_if_absent(pool, premarket_id, user_id)
        .await
        .map_err(ErrorInternalServerError)
}

pub async fn get_users(
    pool: &PgPool,
    premarket_id: Uuid,
    status: Option<WhitelistStatus>,
    cursor: i64,
    limit: i64,
) -> Result<WhitelistUsersResult, actix_web::Error> {
    let res = match status {
        None => whitelist_repo::list_users_by_premarket(pool, premarket_id, cursor, limit).await,
        Some(s) => whitelist_repo::list_users_by_status(pool, premarket_id, s, cursor, limit).await,
    };
    let (items, total) = res.map_err(ErrorInternalServerError)?;


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
) -> Result<u64, actix_web::Error> {
    let uid = match (user_id, user_wallet_address) {
        (Some(id), _) => id,
        (None, Some(addr)) => {
            let user_opt = user_service::get_by_wallet_address(pool, addr)
                .await
                .map_err(ErrorInternalServerError)?;

            match user_opt {
                Some(u) => u.id,
                None => return Ok(0),
            }
        }
        _ => return Err(ErrorBadRequest("user_id or user_wallet_address required")),
    };

    let affected = whitelist_repo::delete_by_premarket_user(pool, premarket_id, uid)
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(affected)
}

pub async fn approve_user(
    pool: &PgPool,
    premarket_id: Uuid,
    user_id: Uuid,
) -> Result<(), actix_web::Error> {
    set_user_status(
        pool,
        premarket_id,
        user_id,
        WhitelistStatus::Approved,
    ).await
}

pub async fn reject_user(
    pool: &PgPool,
    premarket_id: Uuid,
    user_id: Uuid,
) -> Result<(), actix_web::Error> {
    set_user_status(
        pool,
        premarket_id,
        user_id,
        WhitelistStatus::Rejected,
    ).await
}

pub async fn set_user_status(
    pool: &PgPool,
    premarket_id: Uuid,
    user_id: Uuid,
    status: WhitelistStatus,
) -> Result<(), actix_web::Error> {
    whitelist_repo::update_status(pool, premarket_id, user_id, status)
        .await
        .map_err(ErrorInternalServerError)
}

