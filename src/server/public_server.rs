use crate::api::dto::*;
use crate::models::user::WalletAddress;
use crate::services::auth_service;
use crate::services::jwt_service;
use crate::services::public_info_service;
use actix_web::{web, HttpResponse, Scope};
use sqlx::PgPool;

pub fn public_scope() -> Scope {
    web::scope("/auth")
        .route("/start_session", web::get().to(start_session))
        .route("/confirm_login", web::post().to(confirm_login))
        .route("/wallet_info/{pubkey}", web::get().to(wallet_info))
        .route(
            "/premarket_info/{premarket_account}",
            web::get().to(premarket_info),
        )
}

async fn start_session() -> HttpResponse {
    let session = auth_service::generate_session();
    let jwt = jwt_service::create_jwt_handle(&session.nonce);

    let response = StartSessionResponseDto {
        nonce: session.nonce,
        jwt,
    };
    HttpResponse::Ok().json(response)
}

pub async fn confirm_login(
    pool: web::Data<PgPool>,
    req: web::Json<ConfirmLoginRequestDto>,
) -> HttpResponse {
    let handle = match jwt_service::decode_jwt_handle(&req.jwt) {
        Ok(handle) => handle,
        Err(_) => {
            return HttpResponse::Unauthorized().body("no jwt");
        }
    };
    let nonce = handle.nonce;

    let wallet = WalletAddress {
        address: req.wallet_address.clone(),
    };

    if !auth_service::verify_signature(&wallet, &nonce, &req.signature) {
        return HttpResponse::Unauthorized().body("Invalid signature");
    }

    let (user, is_new_user) =
        match auth_service::get_or_create_user_info(pool.get_ref(), &wallet.address).await {
            Ok((user, is_new_user)) => (user, is_new_user),
            Err(err) => {
                tracing::error!("confirm_login user lookup failed: {err:?}");
                return HttpResponse::InternalServerError().body(match err {
                    auth_service::AuthServiceError::FetchUser(_) => "Failed to fetch user",
                    auth_service::AuthServiceError::CreateUser(_) => "Failed to create user",
                });
            }
        };

    let token = jwt_service::create_jwt_with_user(
        user.id,
        &req.wallet_address,
        user.username.clone(),
        user.avatar_url.clone(),
        &nonce,
    );

    let response = ConfirmLoginResponseDto {
        jwt: token,
        is_new_user: is_new_user,
    };
    HttpResponse::Ok().json(response)
}

async fn wallet_info(pubkey: web::Path<String>) -> HttpResponse {
    match public_info_service::get_wallet_info(pubkey.as_str()).await {
        Ok(info) => HttpResponse::Ok().json(WalletInfoResponseDto {
            creation_time: info.creation_time,
            balance: info.balance,
            tx_amount: info.tx_amount,
        }),
        Err(err) => {
            tracing::error!("Error in wallet processing: {err:?}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": err.public_message()
            }))
        }
    }
}

async fn premarket_info(premarket_account_str: web::Path<String>) -> HttpResponse {
    match public_info_service::get_premarket_info(premarket_account_str.as_str()).await {
        Ok(info) => HttpResponse::Ok().json(PremarketInfoResponseDto {
            users: info.users,
            end_timestamp: info.end_timestamp,
            goal_sol: info.goal_sol,
            max_sol: info.max_sol,
            mint: info.mint,
            name: info.name,
            symbol: info.symbol,
            uri: info.uri,
            creator: info.creator,
        }),
        Err(err) => {
            tracing::error!("Error in premarket processing: {err:?}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": err.public_message()
            }))
        }
    }
}
