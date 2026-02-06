// src/server/auth_validation.rs

use actix_web::{HttpRequest, HttpMessage};
use solana_sdk::pubkey::Pubkey;

use crate::api::errors::ApiError;
use crate::services::jwt_service;
use crate::models::premarket::SolanaNetwork;
use crate::models::user::UserContextData;

pub struct BaseRequestContext {
    pub network: SolanaNetwork,
    pub user: UserContextData,
}
pub fn extract_user(req: &HttpRequest) -> Result<UserContextData, ApiError> {
    use std::str::FromStr;

    // ─── JWT ────────────────────────────────────────────────
    let token = req
        .extensions()
        .get::<String>()
        .cloned()
        .unwrap_or_default();

    let token_data = jwt_service::decode_jwt_with_user_info(&token)
        .map_err(|_| ApiError::auth_invalid_token())?;

    let token_pk_str = token_data
        .current_wallet
        .as_deref()
        .ok_or_else(ApiError::auth_missing_wallet)?;

    let token_pk = Pubkey::from_str(token_pk_str)
        .map_err(|_| ApiError::invalid_user_pubkey())?;

    Ok(UserContextData {
        internal_id: token_data.user_id,
        wallets: vec![token_pk],
        current_pubkey: token_pk,
    })
}

pub fn validate_base_request(
    req: &HttpRequest,
    network_str: &str,
    user_pubkey_str: Option<&str>,
) -> Result<BaseRequestContext, ApiError> {
    use std::str::FromStr;

    // ─── NETWORK ────────────────────────────────────────────
    let network = SolanaNetwork::try_from(network_str)
        .map_err(|_| ApiError::invalid_network())?;

    // ─── USER FROM JWT ───────────────────────────────────────
    let user = extract_user(req)?;

    // ─── OPTIONAL USER PUBKEY OVERRIDE CHECK ─────────────────
    if let Some(req_pk_str) = user_pubkey_str {
        // validate request pubkey format
        let req_pk = Pubkey::from_str(req_pk_str)
            .map_err(|_| ApiError::invalid_user_pubkey())?;

        // must match token wallet
        if req_pk != user.current_pubkey {
            return Err(ApiError::wrong_user_pubkey_for_user()); // 403
        }
    }

    Ok(BaseRequestContext { network, user })
}
