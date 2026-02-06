use actix_web::{web, HttpRequest, HttpResponse};
use sqlx::PgPool;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use crate::api::errors::{ApiError, ApiErrorCode, ApiResult, FieldError};
use crate::api::vesting::{UpdateVestingInfoRequest, UpdateVestingInfoResponse};
use crate::services::{premarket_service, vesting_service};
use crate::server::auth_validation::validate_base_request;

pub async fn update_vesting(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<UpdateVestingInfoRequest>,
) -> ApiResult<HttpResponse> {
    let dto = body.into_inner();

    let ctx = validate_base_request(&req, &dto.network, Some(&dto.user_pubkey))?;


    // validate pubkey
    let premarket_pubkey = Pubkey::from_str(&dto.premarket_pubkey)
        .map_err(|_| ApiError::invalid_premarket_pubkey())?
        .to_string();

    // validation -> Err(ApiError), НЕ Ok(...)
    if dto.unlock_at_launch_percent > 100 {
        return Err(ApiError::from_field_errors(vec![FieldError {
            field: "unlock_at_launch_percent",
            code: ApiErrorCode::InvalidPercentage,
            message: "unlock_at_launch_percent must be between 0 and 100",
        }]));
    }

    if dto.enabled && dto.vesting_period_sec == 0 {
        return Err(ApiError::from_field_errors(vec![FieldError {
            field: "vesting_period_sec",
            code: ApiErrorCode::InvalidVestingPeriod,
            message: "vesting_period_sec must be > 0 when enabled=true",
        }]));
    }

    let premarket_info: crate::models::premarket::PremarketInfoServiceModel = premarket_service::get_main_premarket_info(pool.get_ref(), &premarket_pubkey)
        .await
        .map_err(|e| {
            eprintln!(
                "[update_vesting] DB error while loading premarket {}: {:?}",
                premarket_pubkey, e
            );
            ApiError::internal_get_db_error()
        })?
        .ok_or_else(ApiError::missing_premarket)?;

    if premarket_info.creator.id != ctx.user.internal_id {
        return Err(ApiError::forbidden());
    }

    let mint = Pubkey::from_str(&premarket_info.token_info.address)
        .map_err(|_| ApiError::internal_update_db_error())?;

    vesting_service::update_vesting_info(
        pool.get_ref(),
        ctx.network,
        premarket_info.id,
        mint,
        dto.enabled,
        dto.vesting_period_sec as i64,
        dto.unlock_at_launch_percent as i64,
    )
    .await
    .map_err(|e| {
        eprintln!("[update_vesting] failed: {:?}", e);
        ApiError::internal_update_db_error()
    })?;

    Ok(HttpResponse::Ok().json(UpdateVestingInfoResponse { ok: true }))
}
