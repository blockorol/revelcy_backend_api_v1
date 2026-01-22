use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user::User;
use crate::models::whitelist::WhitelistUsersResult;
use crate::services::user_service;
use crate::storage::whitelist_storage;


pub async fn is_user_whitelisted(
    pool: &PgPool,
    premarket_id: Uuid,
    user_id: Uuid,
) -> Result<bool, actix_web::Error> {
    whitelist_storage::exists(pool, premarket_id, user_id)
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

    whitelist_storage::add(pool, premarket_id, uid)
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

    whitelist_storage::add_many(pool, premarket_id, &ids)
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(())
}

/// 4) Получить пользователей whitelist с курсором (offset-based):
/// возвращаем (id, username, avatar_url)
pub async fn get_users(
    pool: &PgPool,
    premarket_id: Uuid,
    cursor: i64,
    limit: i64,
) -> Result<WhitelistUsersResult, actix_web::Error> {
    let (items, total) = whitelist_storage::list_users_by_premarket(pool, premarket_id, cursor, limit)
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(WhitelistUsersResult {
        items,
        total: Some(total),
    })
}
