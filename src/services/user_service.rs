use sqlx::PgPool;

use crate::config;
use crate::models::user::ApplyInviteCodeResult;
use crate::storage::user_repo;
use crate::services::file_service;

use actix_web::{web};
use actix_web::error::ErrorInternalServerError;
use uuid::Uuid;


const GET_IMAGE_PATH: &str = "files/image/";

pub async fn set_invite_code_once(
    pool: web::Data<PgPool>,
    user_id: Uuid,
    invite_code: &str,
) -> Result<ApplyInviteCodeResult, actix_web::Error> {
    user_repo::apply_invite_code_once(pool.get_ref(), user_id, invite_code)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)
}


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
