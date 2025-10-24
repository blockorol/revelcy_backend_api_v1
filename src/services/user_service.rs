use sqlx::PgPool;
use crate::storage::user_repo;
use crate::services::file_service;
use crate::config;
use actix_web::error::ErrorInternalServerError;
use uuid::Uuid;
use crate::models::user::User; 

const GET_IMAGE_PATH: &str = "files/image/";

pub async fn set_username( 
    pool: web::Data<PgPool>,
    user_id: Uuid,
    user_name: &str,
) -> Result<(), actix_web::Error> {
    user_repo::update_username(&pool, user_id, user_name)
        .await
        .map_err(ErrorInternalServerError)
}

pub async fn set_avatar(
    pool: web::Data<PgPool>,
    user_id: Uuid,
    avatar: &[u8],
) -> Result<String, actix_web::Error> {
    let path = file_service::save_png(&user_id.to_string(), avatar)
        .map_err(ErrorInternalServerError)?;
    let host = config::get_host();

    let url = format!("{host}{GET_IMAGE_PATH}{path}");

    user_repo::update_avatar_url(&pool, user_id, &url)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(url)
}

/// Получить полную информацию о пользователях по списку id.
/// Возвращает пустой вектор, если список пуст или пользователи не найдены.
pub async fn get_users_info(
    pool: web::Data<PgPool>,
    user_ids: Vec<Uuid>,
) -> Result<Vec<User>, actix_web::Error> {
    if user_ids.is_empty() {
        return Ok(vec![]);
    }

    let users = user_repo::get_users_by_ids(pool.get_ref(), &user_ids)
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(users)
}
