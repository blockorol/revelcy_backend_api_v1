use std::str::FromStr;
use sqlx::{PgPool};
use uuid::Uuid;
use chrono::Utc;
use actix_web::{web, Error, HttpResponse, HttpRequest, HttpMessage};
use actix_web::error::ErrorInternalServerError;
use crate::api::errors::{ApiErrorCode, ApiError, FieldError, ApiResult};


use solana_sdk::pubkey::Pubkey;
use crate::server::premarket_validation::{
    validate_create_premarket,
    validate_join_premarket,
    validate_extend_premarket,
    validate_finish_premarket,
};
use crate::server::auth_validation::validate_base_request;

use crate::api::premarket::{
    TxToSignRequest,
    JoinPremarketTxRequest, OutPremarketTxRequest, TxOnlyResponse, FinishPremarketTxRequest,
    CreatePremarketTxRequest, CreatePremarketTxResponse,
    GetListQuery, GetListMainInfoDTO, UpdateCommunityDTO,
    FinishedPremarketDTO,
    PremarketTransactionDTO, HolderInfoDTO, UserJoinedToPremarketDTO,
    GetDynamicInfoQuery, TokenDynamicInfoDTO, CreatePremarketDTO, BlockchainInfoDTO,
    CommunityInfoDTO, CommunityLinkDTO, GetMainInfoDTO, GetMainInfoQuery, TokenLinksDTO, TokenState,
    DistributeTokensRequest,
    KillPremarketTxRequest,
    ExtendPremarketTxRequest,
    ExtendedPremarketDTO,
    UpdatePremarketDataDTO,
    DeployTxDTO,
    CheckTxDTO,
    GetHolderEntryPriceQuery, HolderEntryPriceDTO,
    ClaimTokensTxRequest,
    TokenClaimedDTO, TokenClaimedResponse,
};
use crate::models::premarket::{
    BuildFinishTxParams, 
    BuildJoinTxParams, 
    BuildKillTxParams, 
    BuildOutTxParams, 
    BuildPremarketTxParams, 
    BuildClaimTokensTxParams,
    CommunityInfoServiceModel, 
    CommunityLink, DeployTxParams,
    DistributeTokensParams, GetPremarketDataParams,
    HolderInfo, PremarketGoal, PremarketInfoServiceModel,
    PremarketListResult, PremarketState, SolanaNetwork, TokenInfo, 
    TokenLinks, UpdatePremarketDataParams, UserInfoShort,
    CheckTxParams, 
};

use crate::services::{
    jwt_service, premarket_service
};
use crate::middleware::jwt::JwtMiddleware;

use crate::services::solana_service_v2::{
    get_mint_kp,
    build_create_premarket_tx_unsigned,
    parse_create_premarket_tx_from_base64,
    build_extend_premarket_tx_unsigned,
    parse_extend_premarket_tx_from_base64,
    build_join_premarket_tx_unsigned,
    parse_join_premarket_tx_from_base64,
    build_out_premarket_tx_unsigned,
    parse_out_premarket_tx_from_base64,
    build_finish_premarket_tx_unsigned,
    build_claim_tokens_tx_unsigned,
    update_premarket_data_tx_unsigned,
};


use crate::services::solana_service::{
    build_kill_premarket_tx_unsigned,
    check_tx_service, deploy_tx_service,
    distribute_tk,
    get_premarket_data,
    sign_tx_with_revelcy,
    test_build_kill_premarket_tx,
};

pub fn pub_scope() -> impl actix_web::dev::HttpServiceFactory {
    web::scope("/premarket")
        .wrap(JwtMiddleware)
        .route("/get_main_info", web::get().to(get_main_info))
        .route("/get_list", web::get().to(get_list_main_info))
        .route("/get_dynamic_info", web::get().to(get_dynamic_info))
        .route("/get_holder_entry_price", web::get().to(get_holder_entry_price))
        .route("/update_community", web::post().to(update_community_info))

        .route("/created", web::post().to(created_premarket))
        .route("/user_joined", web::post().to(user_joined))
        .route("/user_out", web::post().to(user_out))
        .route("/finished", web::post().to(finished_premarket))
        .route("/killed", web::post().to(killed_premarket))
        .route("/extended_premarket", web::post().to(extended_premarket))
        .route("/distribute_tokens", web::post().to(distribute_tokens))
        .route("/update", web::post().to(update_premarket_data_tx))
        .route("/token_claimed", web::post().to(token_claimed))

        .route("/deploy_tx",   web::post().to(deploy_tx))
        .route("/check_tx",   web::post().to(check_tx))

        .route("/tx/create", web::post().to(create_premarket_tx))
        .route("/tx/join",   web::post().to(join_premarket_tx))
        .route("/tx/out",    web::post().to(out_premarket_tx))
        .route("/tx/finish", web::post().to(finish_premarket_tx))
        .route("/tx/kill",   web::post().to(kill_premarket_tx))
        .route("/tx/extend_premarket", web::post().to(extend_premarket_tx))
        .route("/tx/claim_tokens", web::post().to(claim_tokens_tx))
        
        .route("/tx/test_kill",   web::post().to(test_kill_premarket_tx))

        .route("/tx/sign_create_transaction", web::post().to(sign_transaction))
}

pub async fn sign_transaction(
    req: HttpRequest,
    pool: web::Data<sqlx::PgPool>,
    payload: web::Json<TxToSignRequest>,
) -> ApiResult<HttpResponse> {
    let dto = payload.into_inner();
    let ctx = validate_base_request(&req, dto.network.as_str(), None)?;

    let tx_type = dto.tx_type.as_str();

    // ─────────────────────────────────────────────────────────────
    // 0) validate tx_type
    // ─────────────────────────────────────────────────────────────
    let is_supported = matches!(
        tx_type,
        "create_premarket"
            | "join_premarket"
            | "out_of_premarket"
            | "finish_premarket"
            | "extend_premarket"
            | "claim_tokens"
            | "refund_premarket"
    );

    if !is_supported {
        return Err(ApiError::from_field_errors(vec![FieldError {
            field: "tx_type",
            code: ApiErrorCode::InvalidTxType,
            message: "invalid tx_type",
        }]));
    }

    // ─────────────────────────────────────────────────────────────
    // 1) tx-type specific validation based on unsigned_tx
    //    (parse + validate + user match + sometimes db lookup)
    // ─────────────────────────────────────────────────────────────

    if tx_type == "create_premarket" {
        let parsed = parse_create_premarket_tx_from_base64(&dto.unsigned_tx, ctx.network)
            .map_err(|e| {
                eprintln!("parse create_premarket tx error: {e:?}");
                ApiError::from_field_errors(vec![FieldError {
                    field: "unsigned_tx",
                    code: ApiErrorCode::ValidationError,
                    message: "invalid create_premarket transaction",
                }])
            })?;

        // user in tx must match auth user
        if parsed.params.user != ctx.user_pubkey {
            return Err(ApiError::wrong_user_pubkey_for_user());
        }

        validate_create_premarket(&parsed.params).map_err(ApiError::from_field_errors)?;

        // TODO (later):
        // - optionally, BEFORE signing, run simulation (sigVerify=false) and if failed => ValidationError
        // - ApiErrorCode: ValidationError (field: unsigned_tx) or introduce something like TxSimulationFailed
    }

    if tx_type == "join_premarket" {
        let parsed = parse_join_premarket_tx_from_base64(&dto.unsigned_tx, ctx.network)
            .map_err(|e| {
                eprintln!("parse join_premarket tx error: {e:?}");
                ApiError::from_field_errors(vec![FieldError {
                    field: "unsigned_tx",
                    code: ApiErrorCode::ValidationError,
                    message: "invalid join_premarket transaction",
                }])
            })?;

        if parsed.params.user != ctx.user_pubkey {
            return Err(ApiError::wrong_user_pubkey_for_user());
        }

        validate_join_premarket(&parsed.params).map_err(ApiError::from_field_errors)?;

        // TODO (later):
        // - maybe check premarket state from DB to prevent join when not allowed
        // - if not allowed => ValidationError with a dedicated code
    }

    if tx_type == "out_of_premarket" {
        let parsed = parse_out_premarket_tx_from_base64(&dto.unsigned_tx, ctx.network)
            .map_err(|e| {
                eprintln!("parse out_of_premarket tx error: {e:?}");
                ApiError::from_field_errors(vec![FieldError {
                    field: "unsigned_tx",
                    code: ApiErrorCode::ValidationError,
                    message: "invalid out_of_premarket transaction",
                }])
            })?;

        // user in tx must match auth user
        if parsed.user != ctx.user_pubkey {
            return Err(ApiError::wrong_user_pubkey_for_user());
        }

        // TODO (later):
        // - optionally check in DB that user actually joined and can out
    }

    if tx_type == "extend_premarket" {
        // parse unsigned tx first
        let parsed = parse_extend_premarket_tx_from_base64(&dto.unsigned_tx, ctx.network)
            .map_err(|e| {
                eprintln!("parse extend_premarket tx error: {e:?}");
                ApiError::from_field_errors(vec![FieldError {
                    field: "unsigned_tx",
                    code: ApiErrorCode::ValidationError,
                    message: "invalid extend_premarket transaction",
                }])
            })?;

        if parsed.user != ctx.user_pubkey {
            return Err(ApiError::wrong_user_pubkey_for_user());
        }

        // extend needs DB state validation
        let premarket_str = parsed.premarket.to_string();
        let premarket = premarket_service::get_full_premarket_info(&pool, &premarket_str)
            .await
            .map_err(|e| {
                eprintln!("get_full_premarket_info error: {e:?}");
                // тут это скорее 500 или 400? если “не найден” - 400.
                ApiError::internal_build_tx_failed() // если хочешь точнее: сделать InternalDbFailed
            })?
            .ok_or_else(|| ApiError::missing_premarket())?;

        validate_extend_premarket(
            premarket.main_info.is_extended,
            premarket.main_info.state,
            premarket.main_info.deadline_timestamp,
            parsed.new_deadline,
        )
        .map_err(ApiError::from_field_errors)?;

        // TODO (later):
        // - simulation, then signing, then sending, then cache update
        // - if premarket not found => ApiError::missing_premarket() / invalid_premarket_pubkey()
    }

    if tx_type == "claim_tokens" {
        // TODO: как только у тебя будет parse_claim_tokens_tx_from_base64:
        // - parse unsigned tx
        // - validate user == ctx.user_pubkey
        // - validate token_mint format etc (если нужно)
        // пока можно оставить без parse, но лучше симметрично как с остальными.

        // пример ожидаемой ошибки:
        // return Err(ApiError::from_field_errors(vec![FieldError {
        //   field: "unsigned_tx",
        //   code: ApiErrorCode::ValidationError,
        //   message: "invalid claim_tokens transaction",
        // }]));
    }


    if tx_type == "finish_premarket" {
        let premarket_str = dto.premarket.as_deref().ok_or_else(ApiError::missing_premarket)?;
        let premarket_pub = Pubkey::from_str(premarket_str)
            .map_err(|_| ApiError::invalid_premarket_pubkey())?;

        // 1) DB validation (same as build)
        let full = premarket_service::get_full_premarket_info(pool.get_ref(), premarket_str)
            .await
            .map_err(|e| { eprintln!("get_full_premarket_info error: {e:?}"); ApiError::internal_sign_tx_failed() })?
            .ok_or_else(|| ApiError::from_field_errors(vec![FieldError{
                field: "premarket",
                code: ApiErrorCode::PremarketNotFound,
                message: "premarket not found",
            }]))?;

        validate_finish_premarket(&full)
            .map_err(ApiError::from_field_errors)?;

        // 2) optional: parse tx and ensure correct accounts
        // TODO: parse_finish_premarket_tx_from_base64(&dto.unsigned_tx, ctx.network)
        //   - check parsed.user == ctx.user_pubkey
        //   - check parsed.premarket == premarket_pub

        // 3) load mint key + sign
        let mint_kp = get_mint_kp(pool.get_ref(), premarket_pub).await.map_err(|e| {
            eprintln!("mint_kp load error: {e:?}");
            ApiError::internal_sign_tx_failed()
        })?;

        let extra_signers = vec![mint_kp];
        let signed = sign_tx_with_revelcy(&dto.unsigned_tx, ctx.network, Some(&extra_signers))
            .map_err(|e| { eprintln!("sign_transaction error (finish): {e:?}"); ApiError::internal_sign_tx_failed() })?;

        // TODO (later):
        // - send signed tx to blockchain
        //   on fail => ApiError::internal_send_tx_failed() (лучше отдельный код)
        // - on success => finish handler:
        //   cache_update_finish(premarket_pub, sig, ...)
        // - return signature or transaction depending on API

        return Ok(HttpResponse::Ok().json(TxOnlyResponse { transaction: signed }));
    }

    // ─────────────────────────────────────────────────────────────
    // 2) Default Revelcy sign (no extra signers)
    // ─────────────────────────────────────────────────────────────
    let signed = sign_tx_with_revelcy(&dto.unsigned_tx, ctx.network, None).map_err(|e| {
        eprintln!("sign_transaction error ({tx_type}): {e:?}");
        ApiError::internal_sign_tx_failed()
    })?;

    // TODO (later):
    // - send signed tx to blockchain (per tx_type may choose RPC / options)
    // - if send ok: call tx-type cache handlers:
    //   create_premarket: cache_update_create(...)
    //   join_premarket:   cache_update_join(...)
    //   out_of_premarket: cache_update_out(...)
    //   extend_premarket: cache_update_extend(...)
    //   claim_tokens:     cache_update_claim(...)
    // - decide response: you may want to return signature instead of transaction

    Ok(HttpResponse::Ok().json(TxOnlyResponse { transaction: signed }))
}

pub async fn create_premarket_tx(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    payload: web::Json<CreatePremarketTxRequest>,
) -> ApiResult<HttpResponse> {
    let dto = payload.into_inner();
    
    let ctx = validate_base_request(
        &req,
        dto.network.as_str(),
        Some(dto.user_pubkey.as_str()),
    )?;
    
    let params = BuildPremarketTxParams {
        network: ctx.network,
        user: ctx.user_pubkey,
        name: dto.name,
        symbol: dto.symbol,
        uri: dto.uri,
        deadline: dto.deadline,
        goal: dto.goal_sol_lamp,
        max: dto.max_sol_lamp,
        creator_allocate: dto.creator_allocate_lamp,
    };

    validate_create_premarket(&params)
        .map_err(ApiError::from_field_errors)?;
    


    let res = build_create_premarket_tx_unsigned(pool.get_ref(), params)
        .await
        .map_err(|e| {
            eprintln!("build_create_premarket_tx error: {e:?}");
            ApiError::internal_build_tx_failed()
        })?;

    let body = CreatePremarketTxResponse {
        transaction: res.tx_base64,
        premarket_account_pda: res.premarket_pda.to_string(),
        mint_address: res.mint_address.clone(),
    };

    Ok(HttpResponse::Ok().json(body))

}

pub async fn join_premarket_tx(
    req: HttpRequest,
    payload: web::Json<JoinPremarketTxRequest>,
) -> ApiResult<HttpResponse> {
    let dto = payload.into_inner();

    // 1) jwt + network + сверка user_pubkey с токеном
    let ctx = validate_base_request(
        &req,
        dto.network.as_str(),
        Some(dto.user_pubkey.as_str()),
    )?;

    // 2) premarket pubkey
    if dto.premarket_account.trim().is_empty() {
        return Err(ApiError::missing_premarket());
    }
    let premarket = Pubkey::from_str(dto.premarket_account.as_str())
        .map_err(|_| ApiError::invalid_premarket_pubkey())?;

    // 3) params
    let params = BuildJoinTxParams {
        network: ctx.network,
        user: ctx.user_pubkey,
        premarket,
        amount: dto.amount_sol_lamp,
    };

    // 4) validation
    validate_join_premarket(&params).map_err(ApiError::from_field_errors)?;

    // 5) build tx
    let res = build_join_premarket_tx_unsigned(params)
        .await
        .map_err(|e| {
            eprintln!("build_join_premarket_tx error: {e:?}");
            ApiError::internal_build_tx_failed()
        })?;

    Ok(HttpResponse::Ok().json(TxOnlyResponse {
        transaction: res.tx_base64,
    }))
}

pub async fn out_premarket_tx(
    req: HttpRequest,
    payload: web::Json<OutPremarketTxRequest>,
) -> ApiResult<HttpResponse> {
    let dto = payload.into_inner();

    let ctx = validate_base_request(
        &req,
        dto.network.as_str(),
        Some(dto.user_pubkey.as_str()),
    )?;

    let premarket = Pubkey::from_str(&dto.premarket_account)
        .map_err(|_| ApiError::invalid_premarket_pubkey())?;

    let params = BuildOutTxParams {
        network: ctx.network,
        user: ctx.user_pubkey,
        premarket,
    };

    let res = build_out_premarket_tx_unsigned(params).await.map_err(|e| {
        eprintln!("build_out_premarket_tx error: {e:?}");
        ApiError::internal_build_tx_failed()
    })?;

    Ok(HttpResponse::Ok().json(TxOnlyResponse {
        transaction: res.tx_base64,
    }))
}

pub async fn finish_premarket_tx(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    payload: web::Json<FinishPremarketTxRequest>,
) -> ApiResult<HttpResponse> {
    let dto = payload.into_inner();

    let ctx = validate_base_request(
        &req,
        dto.network.as_str(),
        Some(dto.user_pubkey.as_str()),
    )?;

    let premarket_pub = Pubkey::from_str(&dto.premarket_account)
        .map_err(|_| ApiError::invalid_premarket_pubkey())?;

    // 1) load premarket from DB
    let full = premarket_service::get_full_premarket_info(pool.get_ref(), &dto.premarket_account)
        .await
        .map_err(|e| {
            eprintln!("get_full_premarket_info error: {e:?}");
            ApiError::internal_build_tx_failed() // или отдельный InternalDbFailed
        })?
        .ok_or_else(|| ApiError::from_field_errors(vec![FieldError{
            field: "premarket",
            code: ApiErrorCode::PremarketNotFound,
            message: "premarket not found",
        }]))?;

    // let dynamic = premarket_service::get_dynamic_info(pool.get_ref(), &dto.premarket_account)
    //     .await
    //     .map_err(|e| {
    //         eprintln!("get_dynamic_info error: {e:?}");
    //         ApiError::internal_build_tx_failed()
    //     })?;

    // 3) validate finish business rules
    validate_finish_premarket(&full)
        .map_err(ApiError::from_field_errors)?;

    // 4) build tx
    let params = BuildFinishTxParams {
        network: ctx.network,
        user: ctx.user_pubkey,
        premarket: premarket_pub,
    };

    let res = build_finish_premarket_tx_unsigned(pool.get_ref(), params)
        .await
        .map_err(|e| {
            eprintln!("build_finish_premarket_tx error: {e:?}");
            ApiError::internal_build_tx_failed()
        })?;

    Ok(HttpResponse::Ok().json(TxOnlyResponse {
        transaction: res.tx_base64,
    }))
}

pub async fn distribute_tokens(
    pool: web::Data<PgPool>,
    payload: web::Json<DistributeTokensRequest>,
) -> Result<HttpResponse, Error> {
    let dto = payload.into_inner();

    let network = SolanaNetwork::try_from(dto.network.as_str())
        .map_err(|_| actix_web::error::ErrorBadRequest("invalid network"))?;

    let user = Pubkey::from_str(&dto.user_pubkey)
        .map_err(|_| actix_web::error::ErrorBadRequest("invalid user_pubkey"))?;

    let premarket = Pubkey::from_str(&dto.premarket_account)
        .map_err(|_| actix_web::error::ErrorBadRequest("invalid premarket_account"))?;

    let token_mint = Pubkey::from_str(&dto.token_mint)
        .map_err(|_| actix_web::error::ErrorBadRequest("invalid token_mint"))?;

    let users = dto.users;

    let params = DistributeTokensParams { network, user, premarket, token_mint, users };

    match distribute_tk(&pool, params).await {
        Ok(_) => {
            let msg = format!("Tokens distributet!");
            println!("{}", msg);
            Ok(HttpResponse::Ok().body(msg))
        }
        Err(e) => {
            eprintln!("Error distributing tokens: {:?}", e);
            Ok(HttpResponse::InternalServerError().body("Error distributing tokens"))
        }
    }
}

pub async fn kill_premarket_tx(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    payload: web::Json<KillPremarketTxRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let dto = payload.into_inner();

    let network = SolanaNetwork::try_from(dto.network.as_str())
        .map_err(|_| actix_web::error::ErrorBadRequest("invalid network"))?;

    let user = Pubkey::from_str(&dto.user_pubkey)
        .map_err(|_| actix_web::error::ErrorBadRequest("invalid user_pubkey"))?;

    let _token = req.extensions().get::<String>().cloned();

    let token_data = jwt_service::decode_jwt_with_user_info(&_token.unwrap_or_default()).unwrap();

    match token_data.current_wallet {
        Some(pk) => {
            if pk != dto.user_pubkey {
                println!("user_pubkey does not match token");
                return Ok(HttpResponse::Unauthorized().body("user_pubkey does not match token"));
            } else {
                println!("user_pubkey matches token");
            }
        }
        None => return Ok(HttpResponse::Unauthorized().body("user_pubkey does not match token")),
    }

    let premarket = Pubkey::from_str(&dto.premarket_account)
        .map_err(|_| actix_web::error::ErrorBadRequest("invalid premarket_account"))?;

    let params = GetPremarketDataParams { network, premarket };
    let premarket_data = get_premarket_data(params).await?;

    let users = premarket_data.users
        .into_iter()
        .map(|u| u.wallet.to_string())
        .collect();

    let params = BuildKillTxParams { network, user, premarket, users };

    match build_kill_premarket_tx_unsigned(pool.get_ref(), params).await {
        Ok(res) => Ok(HttpResponse::Ok().json(TxOnlyResponse { transaction: res.tx_base64 })),
        Err(e) => {
            eprintln!("build_kill_premarket_tx error: {e:?}");
            Ok(HttpResponse::InternalServerError().body("failed to build kill tx"))
        }
    }
}

pub async fn test_kill_premarket_tx(
    pool: web::Data<PgPool>,
    payload: web::Json<KillPremarketTxRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let dto = payload.into_inner();

    let network = SolanaNetwork::try_from(dto.network.as_str())
        .map_err(|_| actix_web::error::ErrorBadRequest("invalid network"))?;

    let user = Pubkey::from_str(&dto.user_pubkey)
        .map_err(|_| actix_web::error::ErrorBadRequest("invalid user_pubkey"))?;

    let premarket = Pubkey::from_str(&dto.premarket_account)
        .map_err(|_| actix_web::error::ErrorBadRequest("invalid premarket_account"))?;

    let users = Vec::new(); // GET FROM BLOCKCHAIN

    let params = BuildKillTxParams { network, user, premarket, users };

    match test_build_kill_premarket_tx(pool.get_ref(), params).await {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(e) => {
            eprintln!("build_kill_premarket_tx error: {e:?}");
            Ok(HttpResponse::InternalServerError().body("failed to build kill tx"))
        }
    }
}
pub async fn claim_tokens_tx(
    req: HttpRequest,
    payload: web::Json<ClaimTokensTxRequest>,
) -> ApiResult<HttpResponse> {
    let dto = payload.into_inner();

    let ctx = validate_base_request(
        &req,
        dto.network.as_str(),
        Some(dto.user_pubkey.as_str()),
    )?;

    let premarket = Pubkey::from_str(&dto.premarket_account)
        .map_err(|_| ApiError::invalid_premarket_pubkey())?;

    let token_mint = Pubkey::from_str(&dto.token_mint)
        .map_err(|_| ApiError::invalid_token_mint())?; // добавим

    let params = BuildClaimTokensTxParams {
        network: ctx.network,
        user: ctx.user_pubkey,
        premarket,
        token_mint,
    };

    let res = build_claim_tokens_tx_unsigned(params).await.map_err(|e| {
        eprintln!("build_claim_tokens_tx error: {e:?}");
        ApiError::internal_build_tx_failed()
    })?;

    Ok(HttpResponse::Ok().json(TxOnlyResponse {
        transaction: res.tx_base64,
    }))
}

pub async fn token_claimed(
    pool: web::Data<PgPool>,
    payload: web::Json<TokenClaimedDTO>,
) -> Result<HttpResponse, actix_web::Error> {
    let dto = payload.into_inner();

    let network = SolanaNetwork::try_from(dto.network.as_str())
        .map_err(|_| actix_web::error::ErrorBadRequest("invalid network"))?;

    let premarket = Pubkey::from_str(&dto.premarket_account)
        .map_err(|_| actix_web::error::ErrorBadRequest("invalid premarket_account"))?;

    // Create async RPC client based on network
    let rpc_url = match network {
        SolanaNetwork::Devnet => std::env::var("SOLANA_DEVNET_RPC")
            .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string()),
        SolanaNetwork::MainnetBeta => std::env::var("SOLANA_MAINNET_RPC")
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
    };
    
    let client = solana_client::nonblocking::rpc_client::RpcClient::new(rpc_url);

    // Check on-chain claimed status and update database if needed
    match premarket_service::check_and_update_claimed_status(
        pool.get_ref(),
        &client,
        &premarket,
        &dto.user_pubkey,
    )
    .await
    {
        Ok((claimed, updated_in_db)) => {
            println!(
                "User {} claimed status: {}, DB updated: {}",
                dto.user_pubkey, claimed, updated_in_db
            );
            Ok(HttpResponse::Ok().json(TokenClaimedResponse {
                claimed,
                updated_in_db,
            }))
        }
        Err(e) => {
            eprintln!("token_claimed error: {e:?}");
            Err(e)
        }
    }
}

pub async fn extended_premarket(
    pool: web::Data<sqlx::PgPool>,
    payload: web::Json<ExtendedPremarketDTO>,
) -> Result<HttpResponse, Error> {
    let dto = payload.into_inner();
    
    // Extract premarket pubkey from DTO
    let premarket_pubkey = Pubkey::from_str(&dto.base.premarket_pub_key)
        .map_err(|_| actix_web::error::ErrorBadRequest("invalid premarket_pub_key"))?;
    
    // Parse network
    let network = SolanaNetwork::try_from(dto.network.as_str())
        .map_err(|_| actix_web::error::ErrorBadRequest("invalid network"))?;
    
    // Call get_premarket_data to check extended_premarket flag
    let params = GetPremarketDataParams { network, premarket: premarket_pubkey };
    let premarket_data = get_premarket_data(params).await
        .map_err(|e| {
            eprintln!("❌ Failed to get premarket data: {}", e);
            actix_web::error::ErrorInternalServerError("failed to get premarket data")
        })?;
    
    // Check if extended_premarket is true
    if !premarket_data.extended_premarket {
        return Ok(HttpResponse::BadRequest().body("extended_premarket is not true"));
    }
    
    // Update database with new deadline
    premarket_service::update_premarket_deadline(
        pool.get_ref(),
        &dto.base.premarket_pub_key,
        dto.new_deadline,
    )
    .await
    .map_err(|e| {
        eprintln!(
            "❌ Failed to update premarket '{}' deadline: {} (tx: {}, wallet: {})",
            dto.base.premarket_pub_key,
            e,
            dto.base.tx,
            dto.base.user_wallet
        );
        actix_web::error::ErrorInternalServerError("failed to update premarket deadline")
    })?;

    // Set premarket state to "premarket"
    premarket_service::set_premarket_state(
        pool.get_ref(),
        &dto.base.premarket_pub_key,
        PremarketState::Premarket,
        None,
    )
    .await
    .map_err(|e| {
        eprintln!(
            "❌ Failed to set premarket '{}' state to Premarket: {} (tx: {}, wallet: {})",
            dto.base.premarket_pub_key,
            e,
            dto.base.tx,
            dto.base.user_wallet
        );
        actix_web::error::ErrorInternalServerError("failed to set premarket state")
    })?;

    Ok(HttpResponse::Ok().json("ok"))
}

pub async fn extend_premarket_tx(
    req: HttpRequest,
    payload: web::Json<ExtendPremarketTxRequest>,
    pool: web::Data<PgPool>,
) -> ApiResult<HttpResponse> {
    let dto = payload.into_inner();

    let ctx = validate_base_request(
        &req,
        dto.network.as_str(),
        Some(dto.user_pubkey.as_str()),
    )?;

    let premarket_key = Pubkey::from_str(&dto.premarket_account)
        .map_err(|_| ApiError::invalid_premarket_pubkey())?;

    let premarket = premarket_service::get_full_premarket_info(&pool, &dto.premarket_account)
        .await
        .map_err(|e| {
            eprintln!("❌ Failed to get premarket({}) data: {}", dto.premarket_account, e);
            ApiError::missing_premarket()
        })?
        .ok_or_else(ApiError::missing_premarket)?;

    validate_extend_premarket(
        premarket.main_info.is_extended,
        premarket.main_info.state,
        premarket.main_info.deadline_timestamp,
        dto.new_deadline,
    )
    .map_err(ApiError::from_field_errors)?;

    let res = build_extend_premarket_tx_unsigned(
        ctx.network,
        ctx.user_pubkey,
        premarket_key,
        dto.new_deadline,
    )
    .await
    .map_err(|e| {
        eprintln!("build_extend_premarket_tx error: {e:?}");
        ApiError::internal_build_tx_failed()
    })?;

    Ok(HttpResponse::Ok().json(TxOnlyResponse {
        transaction: res.tx_base64,
    }))
}

pub async fn get_list_main_info(
    pool: web::Data<PgPool>,
    query: web::Query<GetListQuery>,
) -> Result<HttpResponse, Error> {
    let cursor: i64 = i64::from(query.cursor);
    let mut limit: i64 = i64::from(query.limit);
    if limit <= 0 { limit = 50; }
    if limit > 200 { limit = 200; }

    let list_opt = premarket_service::get_list(pool.get_ref(), cursor, limit)
        .await
        .map_err(ErrorInternalServerError)?;

    let list = list_opt.unwrap_or(PremarketListResult {
        items: Vec::new(),
        total: Some(0),
    });

    let premarkets_vec: Vec<BlockchainInfoDTO> = list.items
        .into_iter()
        .map(|premarket_info| BlockchainInfoDTO {
            id: premarket_info.id.map(|id| id.to_string()),
            ipfs_uri: premarket_info.token_info.data_uri.clone(),
            creator_id: premarket_info.creator.id
                .map(|id| id.to_string())
                .unwrap_or_default(),
            creator_address: premarket_info.creator.blockchain_address.clone(),
            premarket_address: premarket_info.blockchain_address.clone(),
            name: premarket_info.token_info.name.clone(),
            description: premarket_info.token_info.description.clone(),
            symbol: premarket_info.token_info.symbol.clone(),
            image_url: premarket_info.token_info.image_url.clone(),
            links: TokenLinksDTO {
                telegram: premarket_info.token_info.links.telegram.clone(),
                twitter:  premarket_info.token_info.links.twitter.clone(),
                web_site: premarket_info.token_info.links.web_site.clone(),
            },
            premarket_is_extended: Some(premarket_info.is_extended),
            premarket_goal_sol_lamp: premarket_info.goal.solana_lamp.to_string(),
            premarket_deadline: premarket_info.deadline_timestamp,
            premarket_created:  premarket_info.created_timestamp,
            premarket_finished: premarket_info.finished_timestamp,
            mint_address: premarket_info.token_info.address.clone(),
            state: match premarket_info.state {
                PremarketState::Premarket => TokenState::Premarket,
                PremarketState::Canceled  => TokenState::Canceled,
                PremarketState::Finished  => TokenState::Finished,
            },
        })
        .collect();

    let resp = GetListMainInfoDTO {
        premarkets: Some(premarkets_vec), 
        total: list.total,                
    };

    Ok(HttpResponse::Ok().json(resp))
}

pub async fn get_main_info(
    pool: web::Data<PgPool>,
    query: web::Query<GetMainInfoQuery>,
) -> HttpResponse {
    println!("received request to get token info");
    let premarket_id_str = query.premarket_id.trim();

    let pubkey = match Pubkey::from_str(premarket_id_str) {
        Ok(pk) => pk,
        Err(_) => return HttpResponse::BadRequest().body("Invalid premarket_id (not a valid pubkey)"),
    };

    let premarket_info = match premarket_service::get_full_premarket_info(&pool, &pubkey.to_string()).await {
        Ok(Some(info)) => info,
        Ok(None) => return HttpResponse::NotFound().body("Premarket info not found"),
        Err(err) => {
            eprintln!("Error fetching premarket info: {:?}", err);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let blockchain_info = BlockchainInfoDTO {
        id: premarket_info.main_info.id.map(|id| id.to_string()),
        ipfs_uri: premarket_info.main_info.token_info.data_uri.clone(),
        creator_id: premarket_info.main_info.creator.id.map(|id| id.to_string()).unwrap_or_default(),
        creator_address: premarket_info.main_info.creator.blockchain_address.clone(),
        premarket_address: premarket_info.main_info.blockchain_address.clone(),
        name: premarket_info.main_info.token_info.name.clone(),
        description: premarket_info.main_info.token_info.description.clone(),
        symbol: premarket_info.main_info.token_info.symbol.clone(),
        image_url: premarket_info.main_info.token_info.image_url.clone(),
        links: TokenLinksDTO {
            telegram: premarket_info.main_info.token_info.links.telegram.clone(),
            twitter: premarket_info.main_info.token_info.links.twitter.clone(),
            web_site: premarket_info.main_info.token_info.links.web_site.clone(),
        },
        premarket_goal_sol_lamp: premarket_info.main_info.goal.solana_lamp.to_string(),
        premarket_deadline: premarket_info.main_info.deadline_timestamp,
        premarket_is_extended: Some(premarket_info.main_info.is_extended),
        premarket_created: premarket_info.main_info.created_timestamp,
        premarket_finished:  premarket_info.main_info.finished_timestamp,
        mint_address: premarket_info.main_info.token_info.address.clone(),
        state: match premarket_info.main_info.state {
            PremarketState::Premarket => TokenState::Premarket,
            PremarketState::Canceled => TokenState::Canceled,
            PremarketState::Finished => TokenState::Finished,
        },
    };
    let dto_links: Option<Vec<CommunityLinkDTO>> = premarket_info.community.links.map(|links| {
        links
            .into_iter()
            .map(|link| CommunityLinkDTO {
                text: link.text,
                url: link.url,
                r#type: link.r#type.into(), // LinkType → LinkTypeDTO
            })
            .collect()
    });

    let community_info = CommunityInfoDTO {
        description: premarket_info.community.description,
        token_banner_url: premarket_info.community.token_banner_url,
        links: dto_links,
    };

    let response = GetMainInfoDTO {
        blockchain_info,
        community_info,
    };

    HttpResponse::Ok().json(response)
}

pub async fn get_dynamic_info(
    pool: web::Data<PgPool>,
    query: web::Query<GetDynamicInfoQuery>,
) -> Result<HttpResponse, Error> {

    let pubkey = match Pubkey::from_str(&query.premarket_id) {
        Ok(pk) => pk,
        Err(_) => return Ok(HttpResponse::BadRequest().body("Invalid premarket_id")),
    };

    
    println!("Fetching TokenDynamicInfo for {}", query.premarket_id);
    
    let premarket_info = match premarket_service::get_dynamic_info(&pool, &pubkey.to_string()).await {
        Ok(info) => info,
        Err(err) => {
            eprintln!("Error fetching premarket info: {:?}", err);
            return Ok(HttpResponse::InternalServerError().finish());
        }
    };
    let holders_dto: Vec<HolderInfoDTO> = premarket_info.holders
        .into_iter()
        .map(|h| HolderInfoDTO {
            id: h.id,
            wallet_address: h.wallet_address,
            join_timestamp: h.join_timestamp,
            icon_url: h.icon_url,
            username: h.username,
            amount_sol_lamp: h.amount_sol_lamp,
            claimed: h.claimed,
        })
        .collect();

    let resp = TokenDynamicInfoDTO {
        holders_count: premarket_info.holders_count,
        current_price_lamp: premarket_info.current_price_lamp,
        reserved_sol_lamp: premarket_info.reserved_sol_lamp,
        change_24h: premarket_info.change_24h,
        holders: holders_dto,
    };

    Ok(HttpResponse::Ok().json(resp))
}

pub async fn get_holder_entry_price(
    pool: web::Data<PgPool>,
    query: web::Query<GetHolderEntryPriceQuery>,
) -> Result<HttpResponse, Error> {
    let pubkey = match Pubkey::from_str(&query.premarket_id) {
        Ok(pk) => pk,
        Err(_) => return Ok(HttpResponse::BadRequest().body("Invalid premarket_id")),
    };

    println!("Fetching entry price for holder {} in premarket {}", query.holder_wallet, query.premarket_id);

    let entry_price = match premarket_service::get_holder_entry_price(
        &pool,
        &pubkey.to_string(),
        &query.holder_wallet,
    ).await {
        Ok(Some(price)) => price,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().body("Holder not found in premarket"));
        }
        Err(err) => {
            eprintln!("Error fetching holder entry price: {:?}", err);
            return Ok(HttpResponse::InternalServerError().finish());
        }
    };

    let resp = HolderEntryPriceDTO {
        entry_price_lamp: entry_price,
    };

    Ok(HttpResponse::Ok().json(resp))
}

pub async fn created_premarket(
    pool: web::Data<PgPool>,
    payload: web::Json<CreatePremarketDTO>,
) -> Result<HttpResponse, Error> {

    let dto = payload.into_inner();
    let info = dto.blockchain_info;
    let community = dto.community_info;
    let solana_lamp = match info.premarket_goal_sol_lamp.parse::<i64>() {
        Ok(val) => val,
        Err(e) => {
            eprintln!("❌ Failed to parse premarket_goal_sol_lamp: {}", e);
            return Err(actix_web::error::ErrorBadRequest("Invalid lamp value"));
        }
    };


    let premarket = PremarketInfoServiceModel {
        id: None,
        token_info: TokenInfo { 
            address: info.mint_address,
            name: info.name,
            description: info.description,
            symbol: info.symbol,
            image_url: info.image_url,
            data_uri: info.ipfs_uri,
            links:  TokenLinks {
                telegram: info.links.telegram,
                twitter: info.links.twitter,
                web_site: info.links.web_site,
            }
        }, 
        creator: UserInfoShort{
            id: Some(Uuid::parse_str(&info.creator_id).unwrap_or_else(|_| Uuid::nil())),
            blockchain_address: info.creator_address,
        },
        state: info.state.into(),
        goal: PremarketGoal{
            solana_lamp: solana_lamp,
        },
        deadline_timestamp: info.premarket_deadline,
        created_timestamp: info.premarket_created,
        blockchain_address: info.premarket_address,
        finished_timestamp: None,
        is_extended: false,
    };


    let service_links: Option<Vec<CommunityLink>> = community.links.map(|links| {
        links
            .into_iter()
            .map(|link| CommunityLink {
                text: link.text,
                url: link.url,
                r#type: link.r#type.into(), // LinkTypeDTO → LinkType
            })
            .collect()
    });


    let community = CommunityInfoServiceModel {
        description: community.description,
        token_banner_url: community.token_banner_url,
        links: service_links
    };

    if let Err(e) = premarket_service::create_full_premarket_info(&pool, premarket, community).await {
        eprintln!("❌ Failed to create full premarket info: {:?}", e);
        return Err(e);
    }
    Ok(HttpResponse::Ok().body("Saved"))
}

pub async fn update_community_info(
    pool: web::Data<PgPool>,
    payload: web::Json<UpdateCommunityDTO>,
) -> Result<HttpResponse, Error> {
    let dto = payload.into_inner();
    let pubkey = dto.premarket_pubkey;
    let community_dto = dto.community_info;
    let service_links: Option<Vec<CommunityLink>> = community_dto.links.map(|links| {
        links
            .into_iter()
            .map(|link| CommunityLink {
                text: link.text,
                url: link.url,
                r#type: link.r#type.into(),
            })
            .collect()
    });

    let community = CommunityInfoServiceModel {
        description: community_dto.description,
        token_banner_url: community_dto.token_banner_url,
        links: service_links
    };

    if let Err(e) = premarket_service::update_community_info(&pool, &pubkey, community).await {
        eprintln!("❌ Failed to create full premarket info: {:?}", e);
        return Err(e);
    }
    Ok(HttpResponse::Ok().body("Saved"))

}

pub async fn user_joined(
    pool: web::Data<PgPool>,
    payload: web::Json<UserJoinedToPremarketDTO>,
) -> Result<HttpResponse, Error> {
        println!("received request to user_joined");

    let dto = payload.into_inner();
    
    println!("amount: {}", dto.join_amount_in_sol_lamport);
    println!("premarket_pub_key: {}", dto.base.tx);
    println!("premarket_pub_key: {}", dto.base.premarket_pub_key);
    println!("user_wallet: {}", dto.base.user_wallet);
    println!("user_id: {:?}", dto.base.user_id); // Option<Uuid> — лучше с {:?}

    let holder = HolderInfo {
        wallet_address: dto.base.user_wallet,
        id:dto.base.user_id,
        icon_url: None,
        username: None,
        join_timestamp: Utc::now().timestamp_millis(),
        amount_sol_lamp: dto.join_amount_in_sol_lamport,
        claimed: false,
    };
    println!("holder created");

    if let Err(e) = premarket_service::add_holder(&pool, &dto.base.premarket_pub_key, holder).await {
        println!(
            "❌ Failed to add holder to premarket_pubkey {}: {}",
            dto.base.premarket_pub_key,
            e
        );
        return Err(e);
    }
    
    println!("User joined");
    
    Ok(HttpResponse::Ok().body("User joined saved"))
}

pub async fn user_out(
    pool: web::Data<PgPool>,
    payload: web::Json<PremarketTransactionDTO>,
) -> Result<HttpResponse, Error> {
    let dto: PremarketTransactionDTO = payload.into_inner();
    if let Err(e) = premarket_service::remove_holder(&pool, &dto.premarket_pub_key, &dto.user_wallet).await {
        println!(
            "❌ Failed to add holder to premarket_pubkey {}: {}",
            dto.premarket_pub_key,
            e
        );
        return Err(e);
    }
    println!("User out: {:?}", dto);
    Ok(HttpResponse::Ok().body("User out saved"))
}

pub async fn finished_premarket(
    pool: web::Data<sqlx::PgPool>,
    payload: web::Json<FinishedPremarketDTO>,
) -> Result<HttpResponse, Error> {
    let dto = payload.into_inner();
    let new_state   = PremarketState::Finished;
    
    if let Err(e) = premarket_service::set_premarket_state(
        &pool,
        &dto.base.premarket_pub_key,
        new_state,
        Some(Utc::now().timestamp()),
    ).await {
        println!(
            "❌ Failed to set premarket '{}' state to {:?}: {} (tx: {}, wallet: {})",
            dto.base.premarket_pub_key,
            new_state,
            e,
            dto.base.tx,
            dto.base.user_wallet
        );
        return Err(e);
    }

    println!(
        "Premarket '{}' marked as {:?}. tx: {}, wallet: {}",
        dto.base.premarket_pub_key,
        new_state,
        dto.base.tx,
        dto.base.user_wallet
    );

    Ok(HttpResponse::Ok().body("Premarket finished state saved"))
}

pub async fn killed_premarket(
    pool: web::Data<sqlx::PgPool>,
    payload: web::Json<FinishedPremarketDTO>,
) -> Result<HttpResponse, Error> {
    let dto = payload.into_inner();
    let new_state = PremarketState::Canceled;

    if let Err(e) = premarket_service::set_premarket_state(
        &pool,
        &dto.base.premarket_pub_key,
        new_state,
        Some(Utc::now().timestamp())
    ).await {
        println!(
            "❌ Failed to set premarket '{}' state to {:?}: {} (tx: {}, wallet: {})",
            dto.base.premarket_pub_key,
            new_state,
            e,
            dto.base.tx,
            dto.base.user_wallet
        );
        return Err(e);
    }

    println!(
        "Premarket '{}' marked as {:?}. tx: {}, wallet: {}",
        dto.base.premarket_pub_key,
        new_state,
        dto.base.tx,
        dto.base.user_wallet
    );

    Ok(HttpResponse::Ok().body("Premarket finished state saved"))
}

impl From<TokenState> for PremarketState {
    fn from(state: TokenState) -> Self {
        match state {
            TokenState::Premarket => PremarketState::Premarket,
            TokenState::Canceled => PremarketState::Canceled,
            TokenState::Finished => PremarketState::Finished,
        }
    }
}

pub async fn update_premarket_data_tx(
    payload: web::Json<UpdatePremarketDataDTO>,
) -> Result<HttpResponse, Error> {

    let dto = payload.into_inner();

    let params = UpdatePremarketDataParams {
        network: dto.network,
        user_pubkey: dto.user_pubkey,
        premarket_account: dto.premarket_account,
        end_timestamp: dto.end_timestamp,
        end_timestamp_updated: dto.end_timestamp_updated,
        goal_sol: dto.goal_sol,
        max_sol: dto.max_sol,
        mint: dto.mint,
        name: dto.name,
        symbol: dto.symbol,
        uri: dto.uri,
        creator: dto.creator,
    };

    match update_premarket_data_tx_unsigned(params).await {
        Ok(res) => Ok(HttpResponse::Ok().json(TxOnlyResponse { transaction: res.tx_base64 })),
        Err(e) => {
            eprintln!("update_premarket_data error: {e:?}");
            Ok(HttpResponse::InternalServerError().body("failed to build kill tx"))
        }
    }

}

pub async fn deploy_tx(
    pool: web::Data<sqlx::PgPool>,
    payload: web::Json<DeployTxDTO>,
) -> Result<HttpResponse, Error> {

    let dto = payload.into_inner();

    let params = DeployTxParams {
        network: dto.network,
        tx: dto.tx,
    };

    match deploy_tx_service(pool.get_ref(), params).await {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(e) => {
            eprintln!("build_kill_premarket_tx error: {e:?}");
            Ok(HttpResponse::InternalServerError().body("failed to build kill tx"))
        }
    }
}

pub async fn check_tx(
    pool: web::Data<sqlx::PgPool>,
    payload: web::Json<CheckTxDTO>,
) -> Result<HttpResponse, Error> {
    
    let dto = payload.into_inner();

    let params = CheckTxParams {
        network: dto.network,
        sig: dto.sig,
    };

    match check_tx_service(pool.get_ref(), params).await {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(e) => {
            eprintln!("build_kill_premarket_tx error: {e:?}");
            Ok(HttpResponse::InternalServerError().body("failed to build kill tx"))
        }
    }
}