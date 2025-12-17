// src/server/auth_validation.rs

use actix_web::{HttpRequest, HttpMessage};
use solana_sdk::pubkey::Pubkey;

use crate::api::errors::ApiError;
use crate::services::jwt_service;
use crate::models::premarket::SolanaNetwork;

pub struct BaseRequestContext {
    pub network: SolanaNetwork,
    pub user_pubkey: Pubkey,
    pub token_data: jwt_service::TokenWithUserInfo,
}

pub fn validate_base_request(
    req: &HttpRequest,
    network_str: &str,
    user_pubkey_str: Option<&str>,
) -> Result<BaseRequestContext, ApiError> {
    use std::str::FromStr;

    // ─── JWT ────────────────────────────────────────────────
    let token = req
        .extensions()
        .get::<String>()
        .cloned()
        .unwrap_or_default();

    let token_data = jwt_service::decode_jwt_with_user_info(&token)
        .map_err(|_| ApiError::auth_missing_wallet())?;

    // ─── NETWORK ────────────────────────────────────────────
    let network = SolanaNetwork::try_from(network_str)
        .map_err(|_| ApiError::invalid_network())?;

    // ─── USER PUBKEY ────────────────────────────────────────
    let user_pubkey = match (user_pubkey_str, token_data.current_wallet.as_deref()) {
        (Some(req_pk), Some(token_pk)) => {
            if req_pk != token_pk {
                return Err(ApiError::wrong_user_pubkey_for_user()); // 403
            }
            Pubkey::from_str(req_pk)
                .map_err(|_| ApiError::invalid_user_pubkey())?
        }

        (Some(_), None) => {
            return Err(ApiError::auth_missing_wallet()); // 401
        }

        (None, Some(token_pk)) => {
            Pubkey::from_str(token_pk)
                .map_err(|_| ApiError::invalid_user_pubkey())?
        }

        (None, None) => {
            return Err(ApiError::auth_missing_wallet()); // 401
        }
    };

    Ok(BaseRequestContext {
        network,
        user_pubkey,
        token_data,
    })
}
