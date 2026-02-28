use std::str::FromStr;

use actix_web::{web, HttpRequest, HttpResponse};
use sqlx::PgPool;
use solana_sdk::pubkey::Pubkey;

use crate::api::errors::{ApiError, ApiErrorCode,FieldError, ApiResult};
use crate::api::whitelist::{
    WhitelistSetStatusRequest, WhitelistSetStatusResponse,
    RemoveWhitelistUserRequest, RemoveWhitelistUserResponse,
    AddWhitelistUserRequest, AddWhitelistUserListRequest,
    GetWhitelistRequest, GetWhitelistResponse,
    WhitelistUserDTO,
};
use crate::server::auth_validation::validate_base_request;
use crate::services::{premarket_service, user_service, whitelist_service};
use crate::models::user::User;
use crate::models::premarket::PremarketLookupKeyType;
use crate::models::whitelist::WhitelistStatus;
// ─────────────────────────────────────────────────────────────
// Helper: ensure caller is creator of premarket
// ─────────────────────────────────────────────────────────────
async fn ensure_creator(
    pool: &PgPool,
    premarket_id: uuid::Uuid,
    caller_user_id: uuid::Uuid,
) -> Result<(), ApiError> {
    let premarket_info = match premarket_service::get_full_premarket_info(pool, &premarket_id.to_string(), PremarketLookupKeyType::Id)
        .await
        .map_err(|e| {
            eprintln!(
                "[whitelist] Failed to get premarket info for premarket_id={} err={:?}",
                premarket_id, e
            );
            ApiError::internal_get_db_error()
        })? {
            Some(info) => info,
            None => {
                eprintln!("[whitelist] Failed to get premarket info for premarket_id={}", premarket_id);
                return Err(ApiError::missing_premarket());
            }
        };

    if premarket_info.main_info.creator.id != caller_user_id {
        eprintln!(
            "[whitelist] Forbidden: user {} is not creator of premarket {} (creator_id={})",
            caller_user_id, premarket_id, premarket_info.main_info.creator.id
        );
        return Err(ApiError::forbidden());
    }

    Ok(())
}

async fn resolve_user_id_strict(
    pool: &PgPool,
    user_id: Option<uuid::Uuid>,
    user_pubkey: Option<&str>,
) -> Result<Option<uuid::Uuid>, ApiError> {
    if let Some(id) = user_id {
        return Ok(Some(id));
    }

    let Some(pk) = user_pubkey else {
        return Err(ApiError::from_field_errors(vec![FieldError {
            field: "user_id or user_pubkey",
            code: ApiErrorCode::ValidationError,
            message: "user_id or user_pubkey required",
        }]));
    };

    let pk = Pubkey::from_str(pk)
        .map_err(|_| ApiError::invalid_pubkey())?
        .to_string();

    let user_opt = user_service::get_by_wallet_address(pool, &pk)
        .await
        .map_err(|e| {
            eprintln!("[whitelist] user_service::get_by_wallet_address err={:?}", e);
            ApiError::internal_get_db_error()
        })?;

    Ok(user_opt.map(|u| u.id))
}



fn map_user_to_dto(u: User) -> WhitelistUserDTO {
    WhitelistUserDTO {
        id: u.id,
        username: u.username,
        avatar_url: u.avatar_url,
        wallets: u.wallets,
    }
}

// ─────────────────────────────────────────────────────────────
// POST /whitelist/add_user
// ─────────────────────────────────────────────────────────────
pub async fn add_whitelist_user(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    payload: web::Json<AddWhitelistUserRequest>,
) -> ApiResult<HttpResponse> {
    let dto = payload.into_inner();

    let ctx = validate_base_request(&req, &dto.network.to_string(), None)?;

    ensure_creator(pool.get_ref(), dto.premarket_id, ctx.user.internal_id).await?;

    let user_pubkey_opt = match dto.user_pubkey.as_deref() {
        Some(pk) => {
            let pk = Pubkey::from_str(pk)
                .map_err(|_| ApiError::invalid_user_pubkey())? // todo: fix me
                .to_string();
            Some(pk)
        }
        None => None,
    };

    whitelist_service::add_user(
        pool.get_ref(),
        dto.premarket_id,
        dto.user_id,
        user_pubkey_opt.as_deref(),
    )
    .await
    .map_err(|e| {
        eprintln!(
            "[whitelist/add_user] Failed premarket_id={} caller={} user_id={:?} user_pubkey={:?} err={:?}",
            dto.premarket_id, ctx.user.internal_id, dto.user_id, dto.user_pubkey, e
        );
        ApiError::internal_update_db_error()
    })?;

    Ok(HttpResponse::Ok().finish())
}

// ─────────────────────────────────────────────────────────────
// POST /whitelist/add_user_list
// ─────────────────────────────────────────────────────────────
pub async fn add_whitelist_user_list(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    payload: web::Json<AddWhitelistUserListRequest>,
) -> ApiResult<HttpResponse> {
    let dto = payload.into_inner();

    let ctx = validate_base_request(&req, &dto.network.to_string(), None)?;

    // authz: only creator
    ensure_creator(pool.get_ref(), dto.premarket_id, ctx.user.internal_id).await?;

    // validate user_pubkeys (если переданы)
    let user_pubkeys_opt: Option<Vec<String>> = match dto.user_pubkeys {
        Some(list) => {
            let mut out = Vec::with_capacity(list.len());
            for pk in list {
                let s = Pubkey::from_str(&pk)
                    .map_err(|_| ApiError::invalid_user_pubkey())?// todo: fix me
                    .to_string();
                out.push(s);
            }
            Some(out)
        }
        None => None,
    };

    whitelist_service::add_users(
        pool.get_ref(),
        dto.premarket_id,
        dto.user_ids,
        user_pubkeys_opt,
    )
    .await
    .map_err(|e| {
        eprintln!(
            "[whitelist/add_user_list] Failed premarket_id={} caller={} err={:?}",
            dto.premarket_id, ctx.user.internal_id, e
        );
        ApiError::internal_update_db_error()
    })?;

    Ok(HttpResponse::Ok().finish())
}

// ─────────────────────────────────────────────────────────────
// POST /whitelist/get
// ─────────────────────────────────────────────────────────────
pub async fn get_premarket_whitelist(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    payload: web::Json<GetWhitelistRequest>,
) -> ApiResult<HttpResponse> {
    let dto = payload.into_inner();

    let _ctx = validate_base_request(&req, &dto.network.to_string(), None)?;
    let status: Option<WhitelistStatus> = dto.status.map(|s| {
        WhitelistStatus::from_str(&s)
            .map_err(|_| ApiError::from_field_errors(vec![FieldError {
                field: "status",
                code: ApiErrorCode::ValidationError,
                message: "invalid status value".into(),
            }]))
    }).transpose()?;

    let res = whitelist_service::get_users(
        pool.get_ref(),
        dto.premarket_id,
        status,
        dto.cursor,
        dto.limit,
    )
    .await
    .map_err(|e| {
        eprintln!(
            "[whitelist/get] Failed premarket_id={} cursor={} limit={} err={:?}",
            dto.premarket_id, dto.cursor, dto.limit, e
        );
        ApiError::internal_get_db_error()
    })?;

    let response = GetWhitelistResponse {
        items: res.items.into_iter().map(map_user_to_dto).collect(),
        total: res.total,
    };

    Ok(HttpResponse::Ok().json(response))
}

// ─────────────────────────────────────────────────────────────
// POST /whitelist/remove_user
// ─────────────────────────────────────────────────────────────
pub async fn remove_whitelist_user(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    payload: web::Json<RemoveWhitelistUserRequest>,
) -> ApiResult<HttpResponse> {
    let dto = payload.into_inner();
    let ctx = validate_base_request(&req, &dto.network.to_string(), None)?;

    ensure_creator(pool.get_ref(), dto.premarket_id, ctx.user.internal_id).await?;

    let user_pubkey_opt = match dto.user_pubkey.as_deref() {
        Some(pk) => {
            let pk = Pubkey::from_str(pk)
                .map_err(|_| ApiError::invalid_user_pubkey())? // todo fix me
                .to_string();
            Some(pk)
        }
        None => None,
    };

    let removed = whitelist_service::remove_user(
        pool.get_ref(),
        dto.premarket_id,
        dto.user_id,
        user_pubkey_opt.as_deref(),
    )
    .await
    .map_err(|e| {
        eprintln!(
            "[whitelist/remove_user] Failed premarket_id={} caller={} user_id={:?} user_pubkey={:?} err={:?}",
            dto.premarket_id, ctx.user.internal_id, dto.user_id, dto.user_pubkey, e
        );
        ApiError::internal_update_db_error()
    })?;

    Ok(HttpResponse::Ok().json(RemoveWhitelistUserResponse { removed }))
}
// ─────────────────────────────────────────────────────────────
// POST /whitelist/approve_user
// ─────────────────────────────────────────────────────────────
pub async fn whitelist_approve(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    payload: web::Json<WhitelistSetStatusRequest>,
) -> ApiResult<HttpResponse> {
    let dto = payload.into_inner();
    let ctx = validate_base_request(&req, &dto.network.to_string(), None)?;

    ensure_creator(pool.get_ref(), dto.premarket_id, ctx.user.internal_id).await?;

    let uid_opt = resolve_user_id_strict(
        pool.get_ref(),
        dto.user_id,
        dto.user_pubkey.as_deref(),
    )
    .await?;

    let Some(user_id) = uid_opt else {
        return Ok(HttpResponse::Ok().json(WhitelistSetStatusResponse { updated: 0 }));
    };

    whitelist_service::approve_user(
        pool.get_ref(),
        dto.premarket_id,
        user_id,
    )
    .await
    .map_err(|e| {
        eprintln!(
            "[whitelist/approve] Failed premarket_id={} caller={} user_id={} err={:?}",
            dto.premarket_id, ctx.user.internal_id, user_id, e
        );
        ApiError::internal_update_db_error()
    })?;

    Ok(HttpResponse::Ok().json(WhitelistSetStatusResponse { updated: 1 }))
}

pub async fn whitelist_reject(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    payload: web::Json<WhitelistSetStatusRequest>,
) -> ApiResult<HttpResponse> {
    let dto = payload.into_inner();
    let ctx = validate_base_request(&req, &dto.network.to_string(), None)?;

    ensure_creator(pool.get_ref(), dto.premarket_id, ctx.user.internal_id).await?;

    let uid_opt = resolve_user_id_strict(
        pool.get_ref(),
        dto.user_id,
        dto.user_pubkey.as_deref(),
    )
    .await?;

    let Some(user_id) = uid_opt else {
        return Ok(HttpResponse::Ok().json(WhitelistSetStatusResponse { updated: 0 }));
    };

    whitelist_service::reject_user(
        pool.get_ref(),
        dto.premarket_id,
        user_id,
    )
    .await
    .map_err(|e| {
        eprintln!(
            "[whitelist/reject] Failed premarket_id={} caller={} user_id={} err={:?}",
            dto.premarket_id, ctx.user.internal_id, user_id, e
        );
        ApiError::internal_update_db_error()
    })?;

    Ok(HttpResponse::Ok().json(WhitelistSetStatusResponse { updated: 1 }))
}
