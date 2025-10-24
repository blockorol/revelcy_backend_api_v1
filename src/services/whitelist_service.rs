use actix_web::{web, Result as ActixResult};
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use sqlx::PgPool;
use crate::services::user_service;
use uuid::Uuid;
use crate::models::user::User; 


use crate::storage::whitelist_repo;

/// Добавить user в whitelist по premarket_id (UUID как строка).
pub async fn add_user_ids(
    pool: web::Data<PgPool>,
    premarket_id: &Uuid,
    user_ids: Vec<Uuid>,
) -> Result<actix_web::Error> {
    if wallets_address.is_empty() {
        return Ok(());
    }

    whitelist_repo::add_users(&pool, premarket_id, &wallets_address)
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(())
}

/// Добавить user в whitelist по wallet.
pub async fn add_wallets(
    pool: web::Data<PgPool>,
    premarket_id: &Uuid,
    wallets_address: Vec<String>,
) -> Result<actix_web::Error> {
    if wallets_address.is_empty() {
        return Ok(());
    }

    whitelist_repo::add_wallets(&pool, premarket_id, &wallets_address)
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(())
}

/// Проверка доступа по кошельку:
/// true  — если whitelist для premarket_id пуст ИЛИ кошелёк в whitelist
/// false — если whitelist есть, но этого кошелька в нём нет
pub async fn has_user_access(
    pool: web::Data<PgPool>,
    premarket_id: &Uuid,
    user_id: &Uuid,
) -> Result<bool, actix_web::Error> {
    let access = whitelist_repo::has_access_by_wallet(&pool, premarket_id, user_id)
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(access)
}

/// Получить **полную информацию** о пользователях из вайтлиста:
pub async fn get_whitelist_users(
    pool: web::Data<PgPool>,
    premarket_id: &Uuid,
) -> Result<Vec<User>, actix_web::Error> {
    let user_ids = whitelist_repo::get_whitelist(pool.get_ref(), *premarket_id)
        .await
        .map_err(ErrorInternalServerError)?;

    if user_ids.is_empty() {
        return Ok(vec![]);
    }

    let users = user_service::get_users_info(pool.get_ref(), &user_ids)
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(users)
}
