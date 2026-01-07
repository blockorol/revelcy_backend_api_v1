// src/server/auth_validation.rs

use actix_web::{HttpRequest, HttpMessage};
use solana_sdk::pubkey::Pubkey;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use crate::api::errors::ApiError;
use crate::services::jwt_service;
use crate::models::premarket::SolanaNetwork;
use crate::models::user::UserContextData;

pub struct BaseRequestContext {
    pub network: SolanaNetwork,
    pub user: UserContextData,
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
        .map_err(|_| ApiError::auth_invalid_token())?;
    // todo: add expired token validation!

    // ─── NETWORK ────────────────────────────────────────────
    let network = SolanaNetwork::try_from(network_str)
        .map_err(|_| ApiError::invalid_network())?;
    

    // ─── USER PUBKEY ────────────────────────────────────────
    // to do: change this validation to check by user_id (from token), is wallet from the users or not
    // 403 - wallet not from the list
    // 401 - no wallet in the request
    // 400 - invalid format user_pubkey_str

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

    let user_context_data = UserContextData {
        internal_id: token_data.user_id,
        wallets: vec![user_pubkey], 
        current_pubkey: user_pubkey,
    };

    Ok(BaseRequestContext {
        network,
        user: user_context_data,
    })
}
