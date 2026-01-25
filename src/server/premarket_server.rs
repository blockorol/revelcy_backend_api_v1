use std::str::FromStr;
use std::time::Duration;
use awc::cookie::time;
use sqlx::{PgPool};
use chrono::Utc;
use actix_web::{web, Error, HttpResponse, HttpRequest, HttpMessage};
use actix_web::error::ErrorInternalServerError;
use crate::api::errors::{ApiErrorCode, ApiError, FieldError, ApiResult};
use crate::constants::{VIRTUAL_SUPPLY_RATIO, VIRTUAL_TOKEN_RATIO};


use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use crate::server::premarket_validation::{
    validate_create_premarket_base,
    validate_create_premarket, validate_extend_premarket,
    validate_finish_premarket, validate_join_premarket,
    validate_refund_premarket, validate_update_uri_premarket,
    validate_withdraw_vesting, 
};
use crate::server::auth_validation::{validate_base_request, BaseRequestContext};

use crate::api::premarket::{
    CreatePremarketConceptRequest, CreatePremarketConceptResponse,
    WithdrawVestingTxRequest, AvailabilityInfoDTO, BlockchainInfoDTO, 
    ClaimTokensTxRequest, CommunityInfoDTO, CommunityLinkDTO, CreatePremarketTxRequest, 
    CreatePremarketTxResponse, UpdateURITxRequest, ExtendPremarketTxRequest, FinishPremarketTxRequest,
     GetDynamicInfoQuery, GetHolderEntryPriceQuery, GetListMainInfoDTO,
     GetListQuery, GetMainInfoDTO, GetMainInfoQuery, HolderEntryPriceDTO,
     HolderInfoDTO, JoinPremarketTxRequest, KillPremarketTxRequest, OutPremarketTxRequest,
     SentTxResponse, ShortPremarketInfoDTO, TokenDynamicInfoDTO, TokenLinksDTO, TokenState,
     TransactionStatus, TxOnlyResponse, TxToSignRequest, UpdateAvailabilityInfoDTO,
     UpdateCommunityDTO,
     OldTxFields, CommonTxFields, Network
};
use crate::models::premarket::{
    CreatePremarketConceptModel,
    BuildClaimTokensTxParams, BuildFinishTxParams, BuildJoinTxParams, BuildKillTxParams, BuildOutTxParams, BuildPremarketTxParams, BuildWithdrawVestingTxParams, CommunityInfoServiceModel, CommunityLink, GetPremarketDataParams, HolderInfo, PremarketGoal, PremarketListResult, PremarketLookupKeyType, PremarketState, SolanaNetwork, TokenInfo, TokenLinks, UserInfoShort
};

use crate::services::{
    jwt_service, premarket_service, ipfs_service, vesting_service
};
use crate::storage::{premarket_repo, vesting_repo};
use crate::middleware::jwt::JwtMiddleware;
use crate::services::background_finaliser::{
    background_finalize_action, 
    UpdateFn
};

use crate::services::solana_service_v2::{
    build_kill_premarket_tx_unsigned,
    build_claim_tokens_tx_unsigned,
    build_create_premarket_tx_unsigned, parse_create_premarket_tx_from_base64,
    build_extend_premarket_tx_unsigned, parse_extend_premarket_tx_from_base64,
    build_update_uri_premarket_tx_unsigned, parse_update_uri_premarket_tx_from_base64,
    build_finish_premarket_tx_unsigned,
    build_join_premarket_tx_unsigned, parse_join_premarket_tx_from_base64,
    build_out_premarket_tx_unsigned, parse_out_premarket_tx_from_base64,
    build_withdraw_vesting_tx_unsigned, parse_withdraw_vesting_tx_from_base64,
    get_mint_kp, send_signed_tx_base64,
    wait_for_confirmed,sign_tx_with_revelcy
};

type ExtraSigners = Option<Vec<solana_sdk::signature::Keypair>>;

pub struct TxFinalizePlan {
    pub tx_type: &'static str,
    pub extra_signers: ExtraSigners,
    pub update_method: UpdateFn,
}

use crate::services::solana_service::{
    get_premarket_data,
};

use crate::server::whitelist_handlers::{
    add_whitelist_user,
    add_whitelist_user_list,
    get_premarket_whitelist,
    remove_whitelist_user,
    whitelist_approve, 
    whitelist_reject,
};

pub fn pub_scope() -> impl actix_web::dev::HttpServiceFactory {
    web::scope("/premarket")
        .wrap(JwtMiddleware)
        .route("/get_main_info", web::get().to(get_main_info))
        .route("/get_list", web::get().to(get_list_main_info))
        .route("/get_dynamic_info", web::get().to(get_dynamic_info))
        .route("/get_holder_entry_price", web::get().to(get_holder_entry_price))

        .route("/update_community", web::post().to(update_community_info))
        .route("/update_availability", web::post().to(update_availability))
        // Whitelist routes: todo: move to separate file
        .route("/whitelist/add_user", web::post().to(add_whitelist_user))
        .route("/whitelist/add_user_list", web::post().to(add_whitelist_user_list))
        .route("/whitelist/get", web::post().to(get_premarket_whitelist))
        .route("/whitelist/remove_user", web::post().to(remove_whitelist_user))
        .route("/whitelist/approve", web::post().to(whitelist_approve))
        .route("/whitelist/reject", web::post().to(whitelist_reject))

        .route("/concept/create", web::post().to(create_concept))
       
        // tx route: todo: move to separate file
        .route("/tx/create", web::post().to(create_premarket_tx))
        .route("/tx/join",   web::post().to(join_premarket_tx))
        .route("/tx/out",    web::post().to(out_premarket_tx))
        .route("/tx/finish", web::post().to(finish_premarket_tx))
        .route("/tx/kill",   web::post().to(kill_premarket_tx))
        .route("/tx/extend_premarket", web::post().to(extend_premarket_tx))
        .route("/tx/update_uri", web::post().to(update_uri_tx))
        .route("/tx/claim_tokens", web::post().to(claim_tokens_tx))
        .route("/tx/withdraw_vesting", web::post().to(withdraw_vesting_tx))

        .route("/tx/sign_and_send_transaction", web::post().to(sign_and_send_transaction))
}

pub async fn create_concept(
    req: HttpRequest,
    pool: web::Data<sqlx::PgPool>,
    payload: web::Json<CreatePremarketConceptRequest>,
) -> ApiResult<HttpResponse> {
    let dto = payload.into_inner();

    let ctx = validate_base_request(&req, &dto.network, Some(&dto.user_pubkey))?;

    validate_create_premarket_base(dto.token_info.deadline, dto.token_info.goal_sol_lamp, dto.token_info.creator_allocate_lamp);

    let create_concept_model = CreatePremarketConceptModel {
        goal: PremarketGoal{
            solana_lamp: dto.token_info.goal_sol_lamp.try_into().unwrap(),
        },
        deadline_timestamp: dto.token_info.deadline,
        created_timestamp: Utc::now().timestamp(),
        is_hided: false,
        short_url_name: None,
        creator: UserInfoShort{
            id: ctx.user.internal_id,
            blockchain_address:ctx.user.current_pubkey.to_string(),
        },
        token_info: TokenInfo {
            address: "".to_string(), // will be set in service
            name: dto.token_info.name,
            description: dto.token_info.description,
            symbol: dto.token_info.symbol,
            image_url: Some(dto.token_info.image_url),
            data_uri: "".to_string(),  // will be set in create tx
            links: TokenLinks {
                telegram: dto.token_info.links.telegram,
                twitter: dto.token_info.links.twitter,
                web_site: dto.token_info.links.web_site,
            },
        },
    };
    let (premarket_id, premarket_pda) = 
        premarket_service::create_concept(&pool, ctx.network, &create_concept_model)
            .await.map_err(|e| {
                eprintln!("create_concept error ({}): {e:?}", dto.user_pubkey);
                ApiError::internal_server_error()
            })?;

    Ok(HttpResponse::Ok().json(CreatePremarketConceptResponse{
        premarket_account_pda: premarket_pda, 
        premarket_id: premarket_id,
    }))
}

pub async fn sign_and_send_transaction(
    req: HttpRequest,
    pool: web::Data<sqlx::PgPool>,
    payload: web::Json<TxToSignRequest>,
) -> ApiResult<HttpResponse> {
    let dto = payload.into_inner();

    // 0) auth + network
    let network_str = match dto.common().network {
        Network::Devnet => "devnet",
        Network::MainnetBeta => "mainnet-beta",
    };
    let ctx = validate_base_request(&req, network_str, None)?;

    let unsigned_tx = dto.common().unsigned_tx.clone();

    // 1) per-tx handling (parse/validate + prepare finalize plan)
    let plan = match dto {
        TxToSignRequest::CreatePremarket(x) => {
            handle_create_premarket(pool.clone(), &ctx, x).await?
        }
        TxToSignRequest::JoinPremarket(x) => handle_join_premarket(pool.clone(), &ctx, x).await?,
        TxToSignRequest::OutOfPremarket(x) => handle_out_of_premarket(pool.clone(), &ctx, x).await?,
        TxToSignRequest::ExtendPremarket(x) => handle_extend_premarket(pool.clone(), &ctx, x).await?,
        TxToSignRequest::UpdateUri(x) => handle_update_uri(pool.clone(), &ctx, x).await?,
        TxToSignRequest::ClaimTokens(x) => handle_claim_tokens(pool.clone(), &ctx, x).await?,
        TxToSignRequest::FinishPremarket(x) => handle_finish_premarket(pool.clone(), &ctx, x).await?,
        TxToSignRequest::RefundPremarket(x) => handle_refund_premarket(pool.clone(), &ctx, x).await?,
        TxToSignRequest::WithdrawVesting(x) => handle_withdraw_vesting(pool.clone(), &ctx, x).await?,
    };

    // 2) sign + send + wait
    let signed = sign_tx_with_revelcy(&unsigned_tx, ctx.network, plan.extra_signers.as_deref())
        .map_err(|e| {
            eprintln!("sign_transaction error ({}): {e:?}", plan.tx_type);
            ApiError::internal_sign_tx_failed()
        })?;

    let sig = send_signed_tx_base64(ctx.network, &signed).await.map_err(|e| {
        eprintln!("send_signed_tx_base64 error ({}): {e:?}", plan.tx_type);
        ApiError::internal_send_tx_failed()
    })?;

    wait_for_confirmed(
        ctx.network,
        &sig,
        Duration::from_secs(60),
        Duration::from_millis(300),
    )
    .await
    .map_err(|e| {
        eprintln!("wait_for_confirmed error ({}): {e:?}", plan.tx_type);
        ApiError::internal_confirm_tx_failed()
    })?;

    // 3) finalize in background
    background_finalize_action(
        ctx.network,
        sig,
        Duration::from_secs(20 * 60),
        Duration::from_secs(1),
        plan.update_method,
    );

    Ok(HttpResponse::Ok().json(SentTxResponse {
        signature: sig.to_string(),
        status: TransactionStatus::Confirmed,
    }))
}

async fn handle_create_premarket(
    pool: web::Data<sqlx::PgPool>,
    ctx: &BaseRequestContext,
    common: CommonTxFields,
) -> ApiResult<TxFinalizePlan> {
    let tx_type: &'static str = "create_premarket";

    let parsed = parse_create_premarket_tx_from_base64(&common.unsigned_tx, ctx.network)
        .map_err(|e| {
            eprintln!("parse create_premarket tx error: {e:?}");
            ApiError::from_field_errors(vec![FieldError {
                field: "unsigned_tx",
                code: ApiErrorCode::ValidationError,
                message: "invalid create_premarket transaction",
            }])
        })?;

    if parsed.params.user != ctx.user.current_pubkey {
        return Err(ApiError::wrong_user_pubkey_for_user());
    }
    println!("parsed premarket_pda: {}", parsed.premarket_pda.to_string());
    println!("parsed mint: {}", parsed.mint.to_string());
    println!("parsed revelcy_auth: {}", parsed.revelcy_auth.to_string());
    println!("parsed user: {}", parsed.user.to_string());
    println!("parsed params.user: {}", parsed.params.user.to_string());

    let premarket_info = match premarket_service::get_main_premarket_info(&pool, &parsed.premarket_pda.to_string()).await {
        Ok(Some(info)) => info,
        Ok(None) => return Err(ApiError::invalid_premarket_pubkey()),
        Err(err) => {
            eprintln!("Error fetching premarket info: {:?}", err);
            return Err(ApiError::internal_server_error());
        }
    };

    
    let uri = parsed.params.uri.clone();

    validate_create_premarket(&ctx.user.current_pubkey.to_string(), &parsed.params, &premarket_info).map_err(ApiError::from_field_errors)?;

    if uri != premarket_info.token_info.data_uri {
        return Err(ApiError::from_field_errors(vec![FieldError {
            field: "uri",
            code: ApiErrorCode::ValidationError,
            message: "tx and db fields are different",
        }]));
    }
    
    if parsed.params.deadline != premarket_info.deadline_timestamp {
        return Err(ApiError::from_field_errors(vec![FieldError {
            field: "deadline",
            code: ApiErrorCode::ValidationError,
            message: "tx and db fields are different",
        }]));
    }
    let goal_db_u64 = u64::try_from(premarket_info.goal.solana_lamp)
    .map_err(|_| {
        eprint!("DB goal is negative or out of range {:?}", parsed.premarket_pda.to_string());
        ApiError::internal_server_error()
    })?;

    if parsed.params.goal != goal_db_u64 {
        return Err(ApiError::from_field_errors(vec![FieldError {
            field: "goal",
            code: ApiErrorCode::ValidationError,
            message: "tx and db fields are different",
        }]));
    }
    if parsed.params.name != premarket_info.token_info.name {
       return Err(ApiError::from_field_errors(vec![FieldError {
            field: "name",
            code: ApiErrorCode::ValidationError,
            message: "tx and db fields are different",
        }]));
    }
    if parsed.params.symbol != premarket_info.token_info.symbol {
        return Err(ApiError::from_field_errors(vec![FieldError {
            field: "symbol",
            code: ApiErrorCode::ValidationError,
            message: "tx and db fields are different",
        }]));
    }

    let amount_initial_buy_sol_lamp = parsed.params.creator_allocate;
    let mint_str = premarket_info.token_info.address.clone();
    let pda_str = premarket_info.blockchain_address.clone();
    let uri_str = uri.clone();

    let pool2 = pool.clone();

    let update_method: UpdateFn = Box::new(move || {
        let pool2 = pool2.clone();
        let mint_str = mint_str.clone();
        let premarket_pubkey = pda_str.clone();
        let uri_str = uri_str.clone();


        Box::pin(async move {
            if amount_initial_buy_sol_lamp != 0 {
                let holder = HolderInfo {
                    wallet_address: premarket_pubkey.clone(),
                    amount_sol_lamp: amount_initial_buy_sol_lamp,
                    join_timestamp: Utc::now().timestamp_millis(),
                    id: None,
                    icon_url: None,
                    username: None,
                    claimed: false,
                };

                premarket_service::add_holder(pool2.get_ref(), &premarket_pubkey, holder)
                    .await
                    .map_err(|e| {
                        eprintln!(
                            "Failed to update DB: initial holder (pda:{}, sol:{}, uri:{}): {}",
                            premarket_pubkey, amount_initial_buy_sol_lamp, uri_str, e
                        );
                    });
            }

            premarket_service::set_premarket_state(pool2.get_ref(), &premarket_pubkey, PremarketState::Premarket, Utc::now().timestamp())
                .await
                .map_err(|e| {
                    eprintln!(
                        "Failed to update DB: create_full_premarket_info (mint:{}, pda:{}, uri:{}): {}",
                        mint_str, premarket_pubkey, uri_str, e
                    );
                });
        })
    });

    Ok(TxFinalizePlan {
        tx_type,
        extra_signers: None,
        update_method,
    })
}

async fn handle_join_premarket(
    pool: web::Data<sqlx::PgPool>,
    ctx: &BaseRequestContext,
    common: CommonTxFields,
) -> ApiResult<TxFinalizePlan> {
    let tx_type: &'static str = "join_premarket";

    let parsed = parse_join_premarket_tx_from_base64(&common.unsigned_tx, ctx.network)
        .map_err(|e| {
            eprintln!("parse join_premarket tx error: {e:?}");
            ApiError::from_field_errors(vec![FieldError {
                field: "unsigned_tx",
                code: ApiErrorCode::ValidationError,
                message: "invalid join_premarket transaction",
            }])
        })?;

    if parsed.params.user != ctx.user.current_pubkey {
        return Err(ApiError::wrong_user_pubkey_for_user());
    }

    validate_join_premarket(&parsed.params).map_err(ApiError::from_field_errors)?;

    let holder = HolderInfo {
        wallet_address: parsed.user.to_string(),
        id: Some(ctx.user.internal_id),
        icon_url: None,
        username: None,
        join_timestamp: Utc::now().timestamp_millis(),
        amount_sol_lamp: parsed.params.amount,
        claimed: false,
    };

    let pool2 = pool.clone();
    let premarket_str = parsed.premarket.to_string();
    let amount = parsed.params.amount;

    let user_internal_id_str = ctx.user.internal_id.to_string();
    let user_pubkey_str = ctx.user.current_pubkey.to_string();

    let update_method: UpdateFn = Box::new(move || {
        let pool2 = pool2.clone();
        let premarket_str = premarket_str.clone();
        let user_internal_id_str = user_internal_id_str.clone();
        let user_pubkey_str = user_pubkey_str.clone();

        Box::pin(async move {
            premarket_service::add_holder(pool2.get_ref(), &premarket_str, holder)
                .await
                .map_err(|e| {
                    eprintln!(
                        "Failed to update DB: add_holder to premarket {} for {}({}), with {}: {}",
                        premarket_str, user_internal_id_str, user_pubkey_str, amount, e
                    );
                });
        })
    });

    Ok(TxFinalizePlan {
        tx_type,
        extra_signers: None,
        update_method,
    })
}

async fn handle_out_of_premarket(
    pool: web::Data<sqlx::PgPool>,
    ctx: &BaseRequestContext,
    common: CommonTxFields,
) -> ApiResult<TxFinalizePlan> {
    let tx_type: &'static str = "out_of_premarket";

    let parsed = parse_out_premarket_tx_from_base64(&common.unsigned_tx, ctx.network)
        .map_err(|e| {
            eprintln!("parse out_of_premarket tx error: {e:?}");
            ApiError::from_field_errors(vec![FieldError {
                field: "unsigned_tx",
                code: ApiErrorCode::ValidationError,
                message: "invalid out_of_premarket transaction",
            }])
        })?;

    if parsed.user != ctx.user.current_pubkey {
        return Err(ApiError::wrong_user_pubkey_for_user());
    }

    let pool2 = pool.clone();
    let premarket_str = parsed.premarket.to_string();
    let user_pubkey_str = ctx.user.current_pubkey.to_string();
    let user_internal_id_str = ctx.user.internal_id.to_string();

    let update_method: UpdateFn = Box::new(move || {
        let pool2 = pool2.clone();
        let premarket_str = premarket_str.clone();
        let user_pubkey_str = user_pubkey_str.clone();
        let user_internal_id_str = user_internal_id_str.clone();

        Box::pin(async move {
            premarket_service::remove_holder(pool2.get_ref(), &premarket_str, &user_pubkey_str)
                .await
                .map_err(|e| {
                    eprintln!(
                        "Failed to update DB: remove_holder from premarket {} for user {} ({}): {}",
                        premarket_str, user_internal_id_str, user_pubkey_str, e
                    );
                });
        })
    });

    Ok(TxFinalizePlan {
        tx_type,
        extra_signers: None,
        update_method,
    })
}

async fn handle_extend_premarket(
    pool: web::Data<sqlx::PgPool>,
    ctx: &BaseRequestContext,
    old: OldTxFields,
) -> ApiResult<TxFinalizePlan> {
    let tx_type: &'static str = "extend_premarket";

    let parsed = parse_extend_premarket_tx_from_base64(&old.common.unsigned_tx, ctx.network)
        .map_err(|e| {
            eprintln!("parse extend_premarket tx error: {e:?}");
            ApiError::from_field_errors(vec![FieldError {
                field: "unsigned_tx",
                code: ApiErrorCode::ValidationError,
                message: "invalid extend_premarket transaction",
            }])
        })?;

    if parsed.user != ctx.user.current_pubkey {
        return Err(ApiError::wrong_user_pubkey_for_user());
    }

    let premarket_str: String = parsed.premarket.to_string();
    let premarket = premarket_service::get_full_premarket_info(
        &pool,
        &premarket_str,
        PremarketLookupKeyType::BcAddress,
    )
    .await
    .map_err(|e| {
        eprintln!("get_full_premarket_info error: {e:?}");
        ApiError::internal_build_tx_failed()
    })?
    .ok_or_else(|| ApiError::missing_premarket())?;

    validate_extend_premarket(
        premarket.main_info.is_extended,
        premarket.main_info.state,
        premarket.main_info.deadline_timestamp,
        parsed.new_deadline,
    )
    .map_err(ApiError::from_field_errors)?;

    let pool2 = pool.clone();
    let premarket_pubkey = parsed.premarket.to_string();
    let new_deadline = parsed.new_deadline;

    let user_pubkey_str = ctx.user.current_pubkey.to_string();
    let user_internal_id_str = ctx.user.internal_id.to_string();

    let update_method: UpdateFn = Box::new(move || {
        let pool2 = pool2.clone();
        let premarket_pubkey = premarket_pubkey.clone();
        let user_pubkey_str = user_pubkey_str.clone();
        let user_internal_id_str = user_internal_id_str.clone();

        Box::pin(async move {
            premarket_service::update_premarket_deadline(pool2.get_ref(), &premarket_pubkey, new_deadline)
                .await
                .map_err(|e| {
                    eprintln!(
                        "Failed to update DB: update_deadline for premarket '{}' deadline: {} by user {} ({}): {}",
                        premarket_pubkey, new_deadline, user_internal_id_str, user_pubkey_str, e
                    );
                });
        })
    });

    Ok(TxFinalizePlan {
        tx_type,
        extra_signers: None,
        update_method,
    })
}

async fn handle_update_uri(
    _pool: web::Data<sqlx::PgPool>,
    ctx: &BaseRequestContext,
    old: OldTxFields,
) -> ApiResult<TxFinalizePlan> {
    let tx_type: &'static str = "update_uri";

    let parsed = parse_update_uri_premarket_tx_from_base64(&old.common.unsigned_tx, ctx.network)
        .map_err(|e| {
            eprintln!("parse update_uri tx error: {e:?}");
            ApiError::from_field_errors(vec![FieldError {
                field: "unsigned_tx",
                code: ApiErrorCode::ValidationError,
                message: "invalid update_uri transaction",
            }])
        })?;

    if parsed.user != ctx.user.current_pubkey {
        return Err(ApiError::wrong_user_pubkey_for_user());
    }

    // TODO: should be changed asap
    let update_method: UpdateFn = Box::new(move || Box::pin(async move {
        eprintln!("PANIC!!!! no method to update DB info for tx_type update_uri");
    }));

    Ok(TxFinalizePlan {
        tx_type,
        extra_signers: None,
        update_method,
    })
}

async fn handle_claim_tokens(
    pool: web::Data<sqlx::PgPool>,
    ctx: &BaseRequestContext,
    old: OldTxFields,
) -> ApiResult<TxFinalizePlan> {
    let tx_type: &'static str = "claim_tokens";

    let premarket_str: String = old.premarket.clone();
    let premarket_pubkey =
        Pubkey::from_str(&premarket_str).map_err(|_| ApiError::invalid_premarket_pubkey())?;

    let pool2 = pool.clone();
    let user_pubkey_str = ctx.user.current_pubkey.to_string();
    let user_internal_id_str = ctx.user.internal_id.to_string();

    let update_method: UpdateFn = Box::new(move || {
        let pool2 = pool2.clone();
        let premarket_str = premarket_str.clone();
        let user_pubkey_str = user_pubkey_str.clone();
        let user_internal_id_str = user_internal_id_str.clone();

        Box::pin(async move {
            premarket_service::user_claimed_token(pool2.get_ref(), &premarket_pubkey, &user_pubkey_str)
                .await
                .map_err(|e| {
                    eprintln!(
                        "Failed to update DB: claim_token PM {} for user {}({}): {}",
                        premarket_str, user_internal_id_str, user_pubkey_str, e
                    );
                });
        })
    });

    Ok(TxFinalizePlan {
        tx_type,
        extra_signers: None,
        update_method,
    })
}

async fn handle_finish_premarket(
    pool: web::Data<sqlx::PgPool>,
    ctx: &BaseRequestContext,
    old: OldTxFields,
) -> ApiResult<TxFinalizePlan> {
    let tx_type: &'static str = "finish_premarket";

    let premarket_str: String = old.premarket.clone();
    let premarket_pubkey =
        Pubkey::from_str(&premarket_str).map_err(|_| ApiError::invalid_premarket_pubkey())?;

    let full = premarket_service::get_full_premarket_info(
        pool.get_ref(),
        &premarket_str,
        PremarketLookupKeyType::BcAddress,
    )
    .await
    .map_err(|e| {
        eprintln!("get_full_premarket_info error: {e:?}");
        ApiError::internal_sign_tx_failed_goal()
    })?
    .ok_or_else(|| {
        ApiError::from_field_errors(vec![FieldError {
            field: "premarket",
            code: ApiErrorCode::PremarketNotFound,
            message: "premarket not found",
        }])
    })?;

    validate_finish_premarket(&full).map_err(ApiError::from_field_errors)?;

    let mint_kp = get_mint_kp(pool.get_ref(), premarket_pubkey).await.map_err(|e| {
        eprintln!("mint_kp load error: {e:?}");
        ApiError::internal_sign_tx_failed()
    })?;

    let extra_signers = Some(vec![mint_kp]);

    let pool2 = pool.clone();
    let user_pubkey_str = ctx.user.current_pubkey.to_string();
    let user_internal_id_str = ctx.user.internal_id.to_string();

    let update_method: UpdateFn = Box::new(move || {
        let pool2 = pool2.clone();
        let premarket_str = premarket_str.clone();
        let user_pubkey_str = user_pubkey_str.clone();
        let user_internal_id_str = user_internal_id_str.clone();

        Box::pin(async move {
            premarket_service::set_premarket_state(
                pool2.get_ref(),
                &premarket_str,
                PremarketState::Finished,
                Utc::now().timestamp(),
            )
            .await
            .map_err(|e| {
                eprintln!(
                    "Failed to update DB: finish_premarket '{}' by user {}({}): {}",
                    premarket_str, user_internal_id_str, user_pubkey_str, e
                );
            });
        })
    });

    Ok(TxFinalizePlan {
        tx_type,
        extra_signers,
        update_method,
    })
}

async fn handle_refund_premarket(
    pool: web::Data<sqlx::PgPool>,
    ctx: &BaseRequestContext,
    old: OldTxFields,
) -> ApiResult<TxFinalizePlan> {
    let tx_type: &'static str = "refund_premarket";

    let premarket_str: String = old.premarket.clone();
    let full = premarket_service::get_full_premarket_info(&pool, &premarket_str, PremarketLookupKeyType::BcAddress)
        .await
        .map_err(|e| { eprintln!("get_full_premarket_info error: {e:?}"); ApiError::internal_sign_tx_failed() })?
        .ok_or_else(|| ApiError::from_field_errors(vec![FieldError{
            field: "premarket",
            code: ApiErrorCode::PremarketNotFound,
            message: "premarket not found",
        }]))?;

    validate_refund_premarket(&full.main_info).map_err(ApiError::from_field_errors)?;

    let pool2 = pool.clone();
    let user_pubkey_str = ctx.user.current_pubkey.to_string();
    let user_internal_id_str = ctx.user.internal_id.to_string();

    let update_method: UpdateFn = Box::new(move || {
        let pool2 = pool2.clone();
        let premarket_str = premarket_str.clone();
        let user_pubkey_str = user_pubkey_str.clone();
        let user_internal_id_str = user_internal_id_str.clone();

        Box::pin(async move {
            premarket_service::set_premarket_state(
                pool2.get_ref(),
                &premarket_str,
                PremarketState::Canceled,
                Utc::now().timestamp(),
            )
            .await
            .map_err(|e| {
                eprintln!(
                    "Failed to update DB: refund_premarket '{}' by user {}({}): {}",
                    premarket_str, user_internal_id_str, user_pubkey_str, e
                );
            });
        })
    });

    Ok(TxFinalizePlan {
        tx_type,
        extra_signers: None,
        update_method,
    })
}

async fn handle_withdraw_vesting(
    _pool: web::Data<sqlx::PgPool>,
    ctx: &BaseRequestContext,
    common: CommonTxFields,
) -> ApiResult<TxFinalizePlan> {
    let tx_type: &'static str = "withdraw_vesting";

    let parsed = parse_withdraw_vesting_tx_from_base64(&common.unsigned_tx, ctx.network)
        .map_err(|e| {
            eprintln!("parse withdraw_vesting tx error: {e:?}");
            ApiError::from_field_errors(vec![FieldError {
                field: "unsigned_tx",
                code: ApiErrorCode::ValidationError,
                message: "invalid withdraw_vesting transaction",
            }])
        })?;

    if parsed.user != ctx.user.current_pubkey {
        return Err(ApiError::wrong_user_pubkey_for_user());
    }

    validate_withdraw_vesting().map_err(ApiError::from_field_errors)?;

    Ok(TxFinalizePlan {
        tx_type,
        extra_signers: None,
        update_method: Box::new(move || Box::pin(async move {})),
    })
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
    
    let mut premarket_info = match premarket_service::get_main_premarket_info(&pool, &dto.premarket_pubkey).await {
        Ok(Some(info)) => info,
        Ok(None) => return Err(ApiError::invalid_premarket_pubkey()),
        Err(err) => {
            eprintln!("Error fetching premarket info: {:?}", err);
            return Err(ApiError::internal_server_error());
        }
    };
    let premarket_pda = Pubkey::from_str(&dto.premarket_pubkey)
        .map_err(|_| ApiError::invalid_premarket_pubkey())?;

    let uri = dto.uri.clone();
    let image_url = dto.image_url.clone();
    let pm_id = premarket_info.id;
    let update_pm_waiter = premarket_service::update_premarket_uri(&pool, &pm_id, &uri, &image_url);
    let mint = Pubkey::from_str(&premarket_info.token_info.address)
        .map_err(|_| ApiError::invalid_premarket_mint_pubkey())?;

    premarket_info.token_info.data_uri = uri.clone();
    premarket_info.token_info.image_url = Some(image_url.clone());
    let params = BuildPremarketTxParams {
        mint: mint.clone(),
        premarket_pda: premarket_pda.clone(),
        network: ctx.network,
        user: ctx.user.current_pubkey.clone(),
        name: premarket_info.token_info.name.clone(),
        symbol: premarket_info.token_info.symbol.clone(),
        uri: premarket_info.token_info.data_uri.clone(),
        deadline: premarket_info.deadline_timestamp.clone(),
        goal: premarket_info.goal.solana_lamp.try_into().unwrap(),
        max: premarket_info.goal.solana_lamp.try_into().unwrap(),
        creator_allocate: dto.creator_allocate_lamp.clone(),
    };

    validate_create_premarket(&ctx.user.current_pubkey.to_string(), &params, &premarket_info.clone())
        .map_err(ApiError::from_field_errors)?;
    


    let res = build_create_premarket_tx_unsigned(pool.get_ref(), params)
        .await
        .map_err(|e| {
            eprintln!("build_create_premarket_tx error: {e:?}");
            ApiError::internal_build_tx_failed()
        })?;

    
    let parsed = parse_create_premarket_tx_from_base64(&res.tx_base64.clone(), ctx.network.clone())
        .map_err(|e| {
            eprintln!("parse create_premarket tx error: {e:?}");
            ApiError::from_field_errors(vec![FieldError {
                field: "unsigned_tx",
                code: ApiErrorCode::ValidationError,
                message: "invalid create_premarket transaction",
            }])
        })?;

    println!("dto from create premarket_pda: {}", dto.premarket_pubkey.to_string());
    println!("parsed from create premarket_pda: {}", parsed.premarket_pda.to_string());
    println!("parsed from create mint: {}", parsed.mint.to_string());
    println!("parsed from create revelcy_auth: {}", parsed.revelcy_auth.to_string());
    println!("parsed from create user: {}", parsed.user.to_string());
    println!("parsed from create params.user: {}", parsed.params.user.to_string());

    let body = CreatePremarketTxResponse {
        transaction: res.tx_base64,
        premarket_account_pda: res.premarket_pda.to_string(),
        mint_address: res.mint_address.clone(),
    };
    update_pm_waiter.await.map_err(|e| {
            eprintln!("update uri error: {e:?}");
            ApiError::internal_build_tx_failed()
        })?;


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
        user: ctx.user.current_pubkey,
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
        user: ctx.user.current_pubkey,
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
    let premarket_info = premarket_service::get_full_premarket_info(pool.get_ref(), &dto.premarket_account, PremarketLookupKeyType::BcAddress)
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

    // 2) Get vesting info from DB to get vesting_period and init_unlock
    let vesting_info = vesting_service::get_full_vesting_info(
            pool.get_ref(),
            &dto.premarket_account,
            vesting_service::VestingLookupType::PremarketAddress
        )
        .await
        .map_err(|e| {
            eprintln!("get_vesting_info error: {e:?}");
            ApiError::internal_build_tx_failed()
        })?
        .ok_or_else(|| ApiError::from_field_errors(vec![FieldError{
            field: "vesting",
            code: ApiErrorCode::VestingNotFound,
            message: "vesting not found for this premarket",
        }]))?;

    // Calculate vesting timestamps from database values
    let now = Utc::now().timestamp();
    let timestamp_start = now;
    let timestamp_end = now + vesting_info.vesting_period;
    let init_unlock = vesting_info.init_unlock as u64;

    // 3) validate finish business rules
    validate_finish_premarket(&premarket_info, timestamp_start, timestamp_end, init_unlock)
        .map_err(ApiError::from_field_errors)?;

    // 4) build tx
    let params = BuildFinishTxParams {
        network: ctx.network,
        user: ctx.user.current_pubkey,
        premarket: premarket_pub,
        timestamp_start,
        timestamp_end,
        init_unlock,
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
        user: ctx.user.current_pubkey,
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

pub async fn withdraw_vesting_tx(
    req: HttpRequest,
    payload: web::Json<WithdrawVestingTxRequest>,
) -> ApiResult<HttpResponse> {
    let dto = payload.into_inner();

    let ctx = validate_base_request(
        &req,
        dto.network.as_str(),
        Some(dto.user_pubkey.as_str()),
    )?;

    let token_mint = Pubkey::from_str(&dto.token_mint)
        .map_err(|_| ApiError::invalid_token_mint())?;

    // Validate withdraw vesting
    validate_withdraw_vesting().map_err(ApiError::from_field_errors)?;

    let params = BuildWithdrawVestingTxParams {
        network: ctx.network,
        user: ctx.user.current_pubkey,
        token_mint,
    };

    let res: crate::models::premarket::BuiltTx = build_withdraw_vesting_tx_unsigned(params).await.map_err(|e| {
        eprintln!("build_withdraw_vesting_tx error: {e:?}");
        ApiError::internal_build_tx_failed()
    })?;

    Ok(HttpResponse::Ok().json(TxOnlyResponse {
        transaction: res.tx_base64,
    }))
}

pub async fn update_uri_tx(
    req: HttpRequest,
    payload: web::Json<UpdateURITxRequest>,
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

    let premarket = premarket_service::get_full_premarket_info(&pool, &dto.premarket_account, PremarketLookupKeyType::BcAddress)
        .await
        .map_err(|e| {
            eprintln!("❌ Failed to get premarket({}) data: {}", dto.premarket_account, e);
            ApiError::missing_premarket()
        })?
        .ok_or_else(ApiError::missing_premarket)?;
    // let info_from_ipfs = ipfs_service::get_ipfs_token_info(&dto.new_uri).await.map_err(|e| {
    //         eprintln!("parse update uri tx error: failed to upload from IPFS: {e:?}");
    //         ApiError::from_field_errors(vec![FieldError {
    //             field: "uri",
    //             code: ApiErrorCode::ValidationError,
    //             message: "failed to get info from IPFS",
    //         }])
    //     })?;

    validate_update_uri_premarket(
        premarket.main_info.state,
        // &premarket.main_info.token_info,
        // &info_from_ipfs,
    )
    .map_err(ApiError::from_field_errors)?;

    let res = build_update_uri_premarket_tx_unsigned(
        ctx.network,
        ctx.user.current_pubkey,
        premarket_key,
        dto.new_uri,
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

    let premarket = premarket_service::get_full_premarket_info(&pool, &dto.premarket_account, PremarketLookupKeyType::BcAddress)
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
        ctx.user.current_pubkey,
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
    req: HttpRequest,
    pool: web::Data<PgPool>,
    query: web::Query<GetListQuery>,
) -> Result<HttpResponse, Error> {
    let cursor: i64 = i64::from(query.cursor);
    let mut limit: i64 = i64::from(query.limit);
    if limit <= 0 { limit = 50; }
    if limit > 200 { limit = 200; }


    let user_opt = validate_base_request(&req, query.network.as_str(), None)
        .ok()
        .map(|ctx| ctx.user); // <-- берём только user

    let list = premarket_service::get_list(pool.get_ref(), cursor, limit, user_opt)
        .await
        .map_err(ErrorInternalServerError)?
        .unwrap_or(PremarketListResult { items: vec![], total: Some(0) });


    let premarkets_vec: Vec<ShortPremarketInfoDTO> = list.items
        .into_iter()
        .map(|premarket_info| ShortPremarketInfoDTO {
            blockchain_info: BlockchainInfoDTO {
                id: Some(premarket_info.id.to_string()),
                ipfs_uri: premarket_info.token_info.data_uri.clone(),
                creator_id: premarket_info.creator.id.to_string(),
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
                    PremarketState::Concept => TokenState::Concept,
                    PremarketState::Premarket => TokenState::Premarket,
                    PremarketState::Canceled  => TokenState::Canceled,
                    PremarketState::Finished  => TokenState::Finished,
                },
            },
            availability_info: AvailabilityInfoDTO {
                token_short_url_name: premarket_info.short_url_name.clone(),
                is_hided: premarket_info.is_hided,
            },
        })
        .collect();

    Ok(HttpResponse::Ok().json(GetListMainInfoDTO {
        premarkets: Some(premarkets_vec),
        total: list.total,
    }))
}

pub async fn get_main_info(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    query: web::Query<GetMainInfoQuery>,
) -> ApiResult<HttpResponse> {
    let id_opt = query.premarket_id.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let name_opt = query.premarket_name.as_deref().map(str::trim).filter(|s| !s.is_empty());

    let (key, key_type): (String, PremarketLookupKeyType) = if let Some(id) = id_opt {
        let pk = Pubkey::from_str(id).map_err(|_| ApiError::invalid_premarket_pubkey())?;
        (pk.to_string(), PremarketLookupKeyType::BcAddress)
    } else if let Some(name) = name_opt {
        (name.to_string(), PremarketLookupKeyType::Name)
    } else {
        return Err(ApiError::from_field_errors(vec![FieldError{
            field: "premarket_id|premarket_name",
            code: ApiErrorCode::ValidationError,
            message: "either premarket_id or premarket_name is required",
        }]));
    };


    let premarket_info = match premarket_service::get_full_premarket_info(&pool, &key, key_type).await {
        Ok(Some(info)) => info,
        Ok(None) => return Err(ApiError::invalid_premarket_pubkey()),
        Err(err) => {
            eprintln!("Error fetching premarket info: {:?}", err);
            return Err(ApiError::internal_server_error());
        }
    };
    if premarket_info.main_info.state == PremarketState::Concept {
        return Err(ApiError::from_field_errors(vec![FieldError{
                field: "state",
                code: ApiErrorCode::PremarketNotFound,
                message: "premarket not found",
            }]));
    }

    if premarket_info.main_info.is_hided && key_type == PremarketLookupKeyType::BcAddress {
        let ctx = match validate_base_request(&req, query.network.as_str(), None) {
            Ok(v) => v,
            Err(_) => return Err(ApiError::invalid_auth_token()),
        };

        let creator_id_ok = premarket_info.main_info.creator.id == ctx.user.internal_id;

        let creator_pubkey = match Pubkey::from_str(premarket_info.main_info.creator.blockchain_address.as_str()) {
            Ok(pk) => pk,
            Err(e) => {
                eprintln!("Invalid creator blockchain_address in DB: {:?}", e);
                return Err(ApiError::internal_server_error())
            }
        };

        let creator_wallet_ok = creator_pubkey == ctx.user.current_pubkey;

        if (!creator_id_ok) || (!creator_wallet_ok) {
            return Err(ApiError::from_field_errors(vec![FieldError{
                field: "premarket_id|premarket_name",
                code: ApiErrorCode::PremarketNotFound,
                message: "premarket not found",
            }]));
        }
    }

    let blockchain_info = BlockchainInfoDTO {
        id: Some(premarket_info.main_info.id.to_string()),
        ipfs_uri: premarket_info.main_info.token_info.data_uri.clone(),
        creator_id: premarket_info.main_info.creator.id.to_string(),
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
            PremarketState::Concept => TokenState::Concept,
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
    let availability_info = AvailabilityInfoDTO {
        token_short_url_name: premarket_info.main_info.short_url_name.clone(),
        is_hided: premarket_info.main_info.is_hided,
    };

    let response = GetMainInfoDTO {
        blockchain_info,
        community_info,
        availability_info,
    };
    
    Ok(HttpResponse::Ok().json(response))
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

pub async fn update_availability(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    payload: web::Json<UpdateAvailabilityInfoDTO>,
) -> ApiResult<HttpResponse> {
    let dto = payload.into_inner();

    let ctx = validate_base_request(&req, &dto.network, None)?; // network тут не важен, только auth

    // 1) validate pubkey
    let premarket_pubkey = Pubkey::from_str(&dto.premarket_pubkey)
        .map_err(|_| ApiError::invalid_premarket_pubkey())?
        .to_string();

    // 2) load premarket
    let pm = premarket_service::get_full_premarket_info(
        &pool,
        &premarket_pubkey,
        PremarketLookupKeyType::BcAddress,
    )
    .await
    .map_err(|e| {
        eprintln!(
            "[update_availability] DB error while loading premarket {}: {:?}",
            premarket_pubkey, e
        );
        ApiError::internal_update_db_error()
    })?
    .ok_or_else(ApiError::missing_premarket)?;

    // 3) check creator
    if !(pm.main_info.creator.id == ctx.user.internal_id) {
        eprintln!(
            "[update_availability] Forbidden: user {} is not creator of premarket {} (creator_id={:?})",
            ctx.user.internal_id,
            premarket_pubkey,
            pm.main_info.creator.id
        );
        return Err(ApiError::forbidden());
    }

    // 4) save
    premarket_service::update_availability_info(
        pool.get_ref(),
        &premarket_pubkey,
        dto.is_hided,
        dto.is_whitelist_enabled,
        dto.token_short_url_name.clone(),
    )
    .await
    .map_err(|e| {
        eprintln!(
            "[update_availability] Failed to update availability for premarket {} by user {}. \
             is_hided={:?}, token_short_url_name={:?}, error={:?}",
            premarket_pubkey,
            ctx.user.internal_id,
            dto.is_hided,
            dto.token_short_url_name,
            e
        );
        ApiError::internal_update_db_error()
    })?;

    println!(
        "[update_availability] OK premarket={} user={}",
        premarket_pubkey,
        ctx.user.internal_id
    );

    Ok(HttpResponse::Ok().finish())
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

impl From<TokenState> for PremarketState {
    fn from(state: TokenState) -> Self {
        match state {
            TokenState::Concept => PremarketState::Concept,
            TokenState::Premarket => PremarketState::Premarket,
            TokenState::Canceled => PremarketState::Canceled,
            TokenState::Finished => PremarketState::Finished,
        }
    }
}
