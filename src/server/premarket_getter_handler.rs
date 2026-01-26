use std::str::FromStr;
use sqlx::{PgPool};
use solana_sdk::pubkey::Pubkey;
use actix_web::{web, Error, HttpResponse};
use crate::api::premarket::{
    GetHolderEntryInfoQuery, GetHolderEntryInfoResponse, TokenEntryInfo
};


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

    let holder_entry_info_opt = match 
        premarket_service::get_holder_entry_info(&pool, &pubkey.to_string(), &holder_wallet.to_string()).await {
        Ok(info) => info,
        Err(err) => {
            eprintln!("Error fetching premarket info: {:?}", err);
            return Ok(HttpResponse::InternalServerError().finish());
        }
    };
    let resp = match holder_entry_info_opt  {
        Some(holder_entry_info) => GetHolderEntryInfoResponse {
            amount_sol_lamp: holder_entry_info.amount_sol_lamp,
            token: TokenEntryInfo{
                total_dec: holder_entry_info.token.total_dec, 
                vested_dec: holder_entry_info.token.vested_dec, 
                claimed_dec: holder_entry_info.token.claimed_dec, 
            },
            rank: holder_entry_info.rank,
        },
        None => {
            return Ok(HttpResponse::Ok().json({}));
        }
    };

    Ok(HttpResponse::Ok().json(resp))
}
