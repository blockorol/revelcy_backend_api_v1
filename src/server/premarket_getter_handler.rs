use crate::api::premarket::{
    GetHolderEntryDataResponse, GetHolderEntryInfoQuery, GetHolderEntryInfoResponse,
    GetHolderWhitelistResponse, TokenEntryInfo,
};
use actix_web::{web, Error, HttpResponse};
use solana_sdk::pubkey::Pubkey;
use sqlx::PgPool;
use std::str::FromStr;

use crate::services::premarket_service;

pub async fn get_user_entry(
    pool: web::Data<PgPool>,
    query: web::Query<GetHolderEntryInfoQuery>,
) -> Result<HttpResponse, Error> {
    let pubkey = match Pubkey::from_str(&query.premarket_id) {
        Ok(pk) => pk,
        Err(_) => return Ok(HttpResponse::BadRequest().body("Invalid premarket_id")),
    };

    let holder_wallet = match Pubkey::from_str(&query.holder_wallet) {
        Ok(pk) => pk,
        Err(_) => return Ok(HttpResponse::BadRequest().body("Invalid premarket_id")),
    };

    let holder_entry_info = match premarket_service::get_holder_entry_info(
        &pool,
        &pubkey.to_string(),
        &holder_wallet.to_string(),
    )
    .await
    {
        Ok(info) => info,
        Err(err) => {
            tracing::error!("Error fetching premarket info: {:?}", err);
            return Ok(HttpResponse::InternalServerError().finish());
        }
    };
    let resp = GetHolderEntryInfoResponse {
        entry: holder_entry_info
            .entry
            .map(|entry| GetHolderEntryDataResponse {
                amount_sol_lamp: entry.amount_sol_lamp,
                token: TokenEntryInfo {
                    total_dec: entry.token.total_dec,
                    vested_dec: entry.token.vested_dec,
                    claimed_dec: entry.token.claimed_dec,
                },
                rank: entry.rank,
            }),
        whitelist: holder_entry_info
            .whitelist
            .map(|w| GetHolderWhitelistResponse {
                status: w.status,
                updated_at: w.updated_at,
            }),
    };

    Ok(HttpResponse::Ok().json(resp))
}
