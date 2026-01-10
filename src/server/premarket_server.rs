use std::str::FromStr;
use std::time::Duration;
use sqlx::{PgPool};
use uuid::Uuid;
use chrono::Utc;
use actix_web::{web, Error, HttpResponse, HttpRequest, HttpMessage};
use actix_web::error::ErrorInternalServerError;
use crate::api::errors::{ApiErrorCode, ApiError, FieldError, ApiResult};


use solana_sdk::pubkey::Pubkey;
use crate::server::premarket_validation::{
    validate_create_premarket, validate_extend_premarket,
    validate_finish_premarket, validate_join_premarket,
    validate_refund_premarket, validate_update_uri_premarket,
    validate_withdraw_vesting
};
use crate::server::auth_validation::validate_base_request;

use crate::api::premarket::{
    WithdrawVestingTxRequest, AvailabilityInfoDTO, BlockchainInfoDTO, 
    CheckTxDTO, ClaimTokensTxRequest, CommunityInfoDTO, CommunityLinkDTO, CreatePremarketTxRequest, 
    CreatePremarketTxResponse, DeployTxDTO, UpdateURITxRequest, ExtendPremarketTxRequest, FinishPremarketTxRequest,
     GetDynamicInfoQuery, GetHolderEntryPriceQuery, GetListMainInfoDTO,
     GetListQuery, GetMainInfoDTO, GetMainInfoQuery, HolderEntryPriceDTO,
     HolderInfoDTO, JoinPremarketTxRequest, KillPremarketTxRequest, OutPremarketTxRequest,
     SentTxResponse, ShortPremarketInfoDTO, TokenDynamicInfoDTO, TokenLinksDTO, TokenState,
     TransactionStatus, TxOnlyResponse, TxToSignRequest, UpdateAvailabilityInfoDTO,
     UpdateCommunityDTO
};
use crate::models::premarket::{
    PremarketLookupKeyType,
    BuildFinishTxParams, 
    BuildJoinTxParams, 
    BuildKillTxParams, 
    BuildOutTxParams, 
    BuildPremarketTxParams, 
    BuildClaimTokensTxParams,
    BuildWithdrawVestingTxParams,
    CommunityInfoServiceModel, 
    CommunityLink, DeployTxParams,
    GetPremarketDataParams,
    HolderInfo, PremarketGoal, PremarketInfoServiceModel,
    PremarketListResult, PremarketState, SolanaNetwork, TokenInfo, 
    TokenLinks, UserInfoShort,
    CheckTxParams, 
};

use crate::services::{
    jwt_service, premarket_service, ipfs_service
};
use crate::middleware::jwt::JwtMiddleware;
use crate::services::background_finaliser::{
    background_finalize_action, 
    UpdateFn
};

use crate::services::solana_service_v2::{
    build_kill_premarket_tx_unsigned,
    build_claim_tokens_tx_unsigned,
    build_create_premarket_tx_unsigned, parse_create_premarket_tx_from_base64,
    build_update_premarket_data_tx_unsigned,
    build_extend_premarket_tx_unsigned, parse_extend_premarket_tx_from_base64,
    build_update_uri_premarket_tx_unsigned, parse_update_uri_premarket_tx_from_base64,
    build_finish_premarket_tx_unsigned,
    build_join_premarket_tx_unsigned, parse_join_premarket_tx_from_base64,
    build_out_premarket_tx_unsigned, parse_out_premarket_tx_from_base64,
    build_withdraw_vesting_tx_unsigned, parse_withdraw_vesting_tx_from_base64,
    get_mint_kp, send_signed_tx_base64,
    wait_for_confirmed
};


use crate::services::solana_service::{
    check_tx_service, deploy_tx_service,
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
        .route("/update_availability", web::post().to(update_availability))

        // .route("/deploy_tx",   web::post().to(deploy_tx))
        // .route("/check_tx",   web::post().to(check_tx))

        .route("/tx/create", web::post().to(create_premarket_tx))
        .route("/tx/join",   web::post().to(join_premarket_tx))
        .route("/tx/out",    web::post().to(out_premarket_tx))
        .route("/tx/finish", web::post().to(finish_premarket_tx))
        .route("/tx/kill",   web::post().to(kill_premarket_tx))
        .route("/tx/extend_premarket", web::post().to(extend_premarket_tx))
        .route("/tx/update_uri", web::post().to(update_uri_tx))
        .route("/tx/claim_tokens", web::post().to(claim_tokens_tx))
        .route("/tx/withdraw_vesting", web::post().to(withdraw_vesting_tx))
        
        .route("/tx/test_kill",   web::post().to(test_kill_premarket_tx))

        .route("/tx/sign_create_transaction", web::post().to(sign_and_send_transaction))
}

pub async fn sign_and_send_transaction(
    req: HttpRequest,
    pool: web::Data<sqlx::PgPool>,
    payload: web::Json<TxToSignRequest>,
) -> ApiResult<HttpResponse> {
    let dto = payload.into_inner();
    let ctx = validate_base_request(&req, dto.network.as_str(), None)?;

    let tx_type = dto.tx_type.clone();
    let tx_type_for_default = tx_type.clone();

    let mut extra_signers: Option<Vec<solana_sdk::signature::Keypair>> = None;
    let mut update_method: UpdateFn = Box::new(move || {
        let tx_type = tx_type_for_default.clone();
        Box::pin(async move {
            eprintln!("PANIC!!!! no method to update DB info for tx_type: {}", tx_type);
        })
    });

    // ─────────────────────────────────────────────────────────────
    // 0) validate tx_type
    // ─────────────────────────────────────────────────────────────
    let is_supported = matches!(
        tx_type.as_str(),
        "create_premarket"
            | "join_premarket"
            | "out_of_premarket"
            | "finish_premarket"
            | "extend_premarket"
            | "update_uri"
            | "claim_tokens"
            | "refund_premarket"
            | "withdraw_vesting"
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
        if parsed.params.user != ctx.user.current_pubkey {
            return Err(ApiError::wrong_user_pubkey_for_user());
        }

        validate_create_premarket(&parsed.params).map_err(ApiError::from_field_errors)?;
        let uri = parsed.params.uri.clone();


        let info_from_ipfs = ipfs_service::get_ipfs_token_info(&parsed.params.uri).await.map_err(|e| {
                eprintln!("parse create_premarket tx error: failed to upload from IPFS: {e:?}");
                ApiError::from_field_errors(vec![FieldError {
                    field: "uri",
                    code: ApiErrorCode::ValidationError,
                    message: "failed to get info from IPFS",
                }])
            })?;
        let premarket = PremarketInfoServiceModel {
            id: None,
            token_info: TokenInfo { 
                address: parsed.mint.to_string(),
                name: parsed.params.name,
                description: info_from_ipfs.description,
                symbol: parsed.params.symbol,
                image_url: info_from_ipfs.image_url,
                data_uri: uri.clone(),
                links: info_from_ipfs.links
            }, 
            creator: UserInfoShort{
                id: Some(ctx.user.internal_id),
                blockchain_address: parsed.params.user.to_string(),
            },
            state: PremarketState::Premarket,
            goal: PremarketGoal{
                solana_lamp: i64::try_from(parsed.params.goal).map_err(|_| ApiError::internal_build_tx_failed())?,
            },
            deadline_timestamp: parsed.params.deadline,
            created_timestamp: Utc::now().timestamp(), // todo: should be get from block info
            blockchain_address: parsed.premarket_pda.to_string(),
            finished_timestamp: None,
            is_extended: false,
            is_hided: false,
            short_url_name: None,
        };
        // community should be updated on the next step
        let community = CommunityInfoServiceModel{
            description: "".to_string(),
            token_banner_url: None, 
            links: None
        };

        let pool2 = pool.clone();
        let amount_initial_buy_sol_lamp = parsed.params.creator_allocate;
        let premarket2 = premarket.clone();
        let premarket_pubkey = premarket.blockchain_address.clone();
        update_method = Box::new(move || { 
            let pool2 = pool2.clone();
            let amount_initial_buy_sol_lamp = amount_initial_buy_sol_lamp.clone();
            let premarket = premarket2.clone();
            let premarket_pubkey = premarket_pubkey.clone();
            Box::pin(async move {
                premarket_service::create_full_premarket_info(
                    pool2.get_ref(),
                    premarket,
                    community
                ).await.map_err(|e| {
                    eprintln!("Failed to update DB: create_full_premarket_info (mint:{:#}, pda:{:#}, uri:{:#}): {:#}", 
                        parsed.mint, 
                        parsed.premarket_pda,
                        uri,
                        e
                    );
                });
                if amount_initial_buy_sol_lamp != 0 {
                    let holder = HolderInfo {
                        wallet_address: premarket_pubkey.clone(),
                        amount_sol_lamp: amount_initial_buy_sol_lamp,
                        join_timestamp: Utc::now().timestamp_millis(),
                        id: None,
                        icon_url: None,
                        username: None,
                        claimed: false
                    };
                    premarket_service::add_holder(
                        pool2.get_ref(),
                        &premarket_pubkey,
                        holder
                    ).await.map_err(|e| {
                    eprintln!("Failed to update DB: join user in alocate (mint:{:#}, pda:{:#}, sol:{:#}, uri:{:#}): {:#}", 
                        parsed.mint, 
                        parsed.premarket_pda,
                        amount_initial_buy_sol_lamp,
                        uri,
                        e
                    );
                });
                }
            })
        });
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

        if parsed.params.user != ctx.user.current_pubkey {
            return Err(ApiError::wrong_user_pubkey_for_user());
        }

        validate_join_premarket(&parsed.params).map_err(ApiError::from_field_errors)?;
        // TODO (later):
        // - maybe check premarket state from DB to prevent join when not allowed
        // - if not allowed => ValidationError with a dedicated code
        
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
        update_method = Box::new(move || { 
            let pool2 = pool2.clone();
            let premarket_str = premarket_str.clone();
            Box::pin(async move {
            premarket_service::add_holder(
                pool2.get_ref(),
                &premarket_str,
                holder
            ).await.map_err(|e| {
                eprintln!(
                    "Failed to update DB: add_holder to premarket_pubkey {} for {}({}), with {}: {}",
                    premarket_str,
                    ctx.user.internal_id.to_string(),
                    ctx.user.current_pubkey.to_string(),
                    parsed.params.amount,
                    e
                );
            });
        })});
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
        if parsed.user != ctx.user.current_pubkey {
            return Err(ApiError::wrong_user_pubkey_for_user());
        }

        // TODO (later):
        // - optionally check in DB that user actually joined and can out
        let pool2 = pool.clone();
        let premarket_str = parsed.premarket.to_string();
        update_method = Box::new(move || {
            let pool2 = pool2.clone();
            let premarket_str = premarket_str.clone();
            Box::pin(async move {
            premarket_service::remove_holder(
                pool2.get_ref(),
                &premarket_str,
                &ctx.user.current_pubkey.to_string(),
            ).await.map_err(|e| {
                println!(
                    "Failed to update DB: remove_holder from premarket {} for user {} ({}): {}",
                    premarket_str,
                    ctx.user.internal_id.to_string(),
                    ctx.user.current_pubkey.to_string(),
                    e
                );
            });
        })});
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

        if parsed.user != ctx.user.current_pubkey {
            return Err(ApiError::wrong_user_pubkey_for_user());
        }

        // extend needs DB state validation
        let premarket_str: String = parsed.premarket.to_string().clone();
        let premarket = premarket_service::get_full_premarket_info(&pool, &premarket_str, PremarketLookupKeyType::BcAddress)
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
        let pool2 = pool.clone();
        let premarket_pubkey = parsed.premarket.to_string();
        update_method = Box::new(move || { 
            let pool2 = pool2.clone();
            let premarket_str = premarket_str.clone();
            let new_deadline = parsed.new_deadline.clone();
            Box::pin(async move {
            // Update database with new deadline
            premarket_service::update_premarket_deadline(
                pool2.get_ref(),
                &premarket_pubkey,
                new_deadline,
            ).await.map_err(|e| {
                eprintln!(
                    "Failed to update DB: update_deadline for premarket '{}' deadline: {} by user {} ({}): {}",
                    premarket_pubkey,
                    new_deadline,
                    ctx.user.internal_id.to_string(), 
                    ctx.user.current_pubkey.to_string(), 
                    e,
                );
            });
        })});
    }

    if tx_type == "update_uri" {
        // parse unsigned tx first
        let parsed = parse_update_uri_premarket_tx_from_base64(&dto.unsigned_tx, ctx.network)
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

        // extend needs DB state validation
        let premarket_str: String = parsed.premarket.to_string().clone();
        let premarket = premarket_service::get_full_premarket_info(&pool, &premarket_str, PremarketLookupKeyType::BcAddress)
            .await
            .map_err(|e| {
                eprintln!("get_full_premarket_info error: {e:?}");
                // тут это скорее 500 или 400? если “не найден” - 400.
                ApiError::internal_build_tx_failed() // если хочешь точнее: сделать InternalDbFailed
            })?
            .ok_or_else(|| ApiError::missing_premarket())?;
        // let info_from_ipfs = ipfs_service::get_ipfs_token_info(&parsed.new_uri).await.map_err(|e| {
        //         eprintln!("parse create_premarket tx error: failed to upload from IPFS: {e:?}");
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
        // let pool2 = pool.clone();
        // let new_uri = parsed.new_uri.clone();
// 
        // let image_url = premarket.image_url.clone();
        // let data_uri  = info_from_ipfs.data_uri.clone();
        // let telegram  = info_from_ipfs.links.telegram.clone();
        // let twitter   = info_from_ipfs.links.twitter.clone();
        // let web_site  = info_from_ipfs.links.web_site.clone();

        let premarket_pubkey = parsed.premarket.to_string();
        update_method = Box::new(move || { 
            // let pool2 = pool2.clone();
            // let premarket_pubkey = premarket_pubkey.clone();
            // let new_uri = parsed.new_uri.clone();
            // let new_uri = new_uri.clone();
            // let image_url = image_url.clone();
            // let data_uri = data_uri.clone();
            // let telegram = telegram.clone();
            // let twitter = twitter.clone();
            // let web_site: Option<String> = web_site.clone();

            Box::pin(async move {
                println!("PANIC!!!! no method to update DB info for tx_type update url");
            // Update database with new deadline
            // premarket_service::update_premarket_links(
            //     pool2.get_ref(),
            //     &premarket_pubkey,
            //     image_url,
            //     data_uri,
            //     telegram,
            //     twitter,
            //     web_site,
            // ).await.map_err(|e| {
            //     eprintln!(
            //         "Failed to update DB: update links for premarket '{}' deadline: {} by user {} ({}): {}",
            //         premarket_pubkey,
            //         new_uri,
            //         ctx.user.internal_id.to_string(), 
            //         ctx.user.current_pubkey.to_string(), 
            //         e,
            //     );
            // });
        })});
    }

    if tx_type == "claim_tokens" {
        // todo: get tx info!!!
        let premarket_str: String = dto.premarket.clone().ok_or_else(ApiError::missing_premarket)?;
        let premarket_pubkey = Pubkey::from_str(&premarket_str)
            .map_err(|_| ApiError::invalid_premarket_pubkey())?;
        // TODO: как только у тебя будет parse_claim_tokens_tx_from_base64:
        // - parse unsigned tx
        // - validate user == ctx.user.current_pubkey
        // - validate token_mint format etc (если нужно)
        // пока можно оставить без parse, но лучше симметрично как с остальными.

        // пример ожидаемой ошибки:
        // return Err(ApiError::from_field_errors(vec![FieldError {
        //   field: "unsigned_tx",
        //   code: ApiErrorCode::ValidationError,
        //   message: "invalid claim_tokens transaction",
        // }]));
        let pool2 = pool.clone();
        let user_wallet_str = ctx.user.current_pubkey.to_string();
        update_method = Box::new(move || {
            let pool2 = pool2.clone();
            let premarket_str = premarket_str.clone();
            Box::pin(async move {
            let user_wallet_str = user_wallet_str.clone();
            premarket_service::user_claimed_token(
                pool2.get_ref(),
                &premarket_pubkey,
                &user_wallet_str,
            ).await
            .map_err(|e| {
                eprintln!(
                    "Failed to update DB: claim_token PM {} for user {}({}): {}",
                    premarket_pubkey.to_string(),
                    ctx.user.internal_id.to_string(), 
                    ctx.user.current_pubkey.to_string(), 
                    e,
                );
            });
        })});
    }

    if tx_type == "finish_premarket" {
        // todo: get premarket_pub from tx!!!! 
        let premarket_str: String = dto.premarket.clone().ok_or_else(ApiError::missing_premarket)?;
        let premarket_pubkey = Pubkey::from_str(&premarket_str)
            .map_err(|_| ApiError::invalid_premarket_pubkey())?;


        // 1) DB validation (same as build)
        let full = premarket_service::get_full_premarket_info(pool.get_ref(), &premarket_str, PremarketLookupKeyType::BcAddress)
            .await
            .map_err(|e| { eprintln!("get_full_premarket_info error: {e:?}"); ApiError::internal_sign_tx_failed_goal() })?
            .ok_or_else(|| ApiError::from_field_errors(vec![FieldError{
                field: "premarket",
                code: ApiErrorCode::PremarketNotFound,
                message: "premarket not found",
            }]))?;

        validate_finish_premarket(&full)
            .map_err(ApiError::from_field_errors)?;

        // 2) optional: parse tx and ensure correct accounts
        // TODO: parse_finish_premarket_tx_from_base64(&dto.unsigned_tx, ctx.network)
        //   - check parsed.user == ctx.user.current_pubkey
        //   - check parsed.premarket == premarket_pub

        // 3) load mint key + sign
        let mint_kp = get_mint_kp(pool.get_ref(), premarket_pubkey).await.map_err(|e| {
            eprintln!("mint_kp load error: {e:?}");
            ApiError::internal_sign_tx_failed()
        })?;

        let extra = vec![mint_kp];
        extra_signers =  Some(extra);
        let pool2 = pool.clone();
        update_method = Box::new(move || { 
            let pool2 = pool2.clone();
            let premarket_str = premarket_str.clone();
            Box::pin(async move {
            premarket_service::set_premarket_state(
                pool2.get_ref(),
                &premarket_str,
                PremarketState::Finished,
                Some(Utc::now().timestamp()), // to do: change me to time from tx
            ).await.map_err(|e| {
                eprintln!(
                    "Failed to update DB: finish_premarket '{}' state to 'Finished' by user {}({}): {}",
                    premarket_str,
                    ctx.user.internal_id.to_string(),
                    ctx.user.current_pubkey.to_string(),
                    e,
                );
            });
        })});
    }

    // tx/kill
    if tx_type == "refund_premarket" { 
        // todo: get premarket_pub from tx!!!! 
        let premarket_str: String = dto.premarket.clone().ok_or_else(ApiError::missing_premarket)?;

        // 1) DB validation (same as build)
        let full = premarket_service::get_full_premarket_info(&pool, &premarket_str, PremarketLookupKeyType::BcAddress)
            .await
            .map_err(|e| { eprintln!("get_full_premarket_info error: {e:?}"); ApiError::internal_sign_tx_failed() })?
            .ok_or_else(|| ApiError::from_field_errors(vec![FieldError{
                field: "premarket",
                code: ApiErrorCode::PremarketNotFound,
                message: "premarket not found",
            }]))?;

        validate_refund_premarket(&full.main_info)
            .map_err(ApiError::from_field_errors)?;

        // 2) optional: parse tx and ensure correct accounts
        // TODO: parse_finish_premarket_tx_from_base64(&dto.unsigned_tx, ctx.network)
        //   - check parsed.user == ctx.user.current_pubkey
        //   - check parsed.premarket 

        let pool2 = pool.clone();
        update_method = Box::new(move || { 
            let pool2 = pool2.clone();
            let premarket_str = premarket_str.clone();
            Box::pin(async move {
            premarket_service::set_premarket_state(
                pool2.get_ref(),
                &premarket_str,
                PremarketState::Canceled,
                Some(Utc::now().timestamp()), // to do: change me to time from tx
            ).await.map_err(|e| {
                eprintln!(
                    "Failed to update DB: refund_premarket '{}' state to 'Canceled' by user {}({}): {}",
                    premarket_str,
                    ctx.user.internal_id.to_string(),
                    ctx.user.current_pubkey.to_string(),
                    e,
                );
            });
        })});
    }

    if tx_type == "withdraw_vesting" {
        let parsed = parse_withdraw_vesting_tx_from_base64(&dto.unsigned_tx, ctx.network)
            .map_err(|e| {
                eprintln!("parse withdraw_vesting tx error: {e:?}");
                ApiError::from_field_errors(vec![FieldError {
                    field: "unsigned_tx",
                    code: ApiErrorCode::ValidationError,
                    message: "invalid withdraw_vesting transaction",
                }])
            })?;

        // User in tx must match auth user
        if parsed.user != ctx.user.current_pubkey {
            return Err(ApiError::wrong_user_pubkey_for_user());
        }

        // Validate withdraw_vesting business rules
        validate_withdraw_vesting().map_err(ApiError::from_field_errors)?;

        // No database update needed for withdraw_vesting
        // Vesting state is stored on-chain
        update_method = Box::new(move || {
            Box::pin(async move {
                // No-op: withdraw_vesting doesn't require database updates
                // All vesting data is managed on-chain
            })
        });
    }
    // ─────────────────────────────────────────────────────────────
    // 2) Default Revelcy sign (no extra signers)
    // ─────────────────────────────────────────────────────────────
    let signed = sign_tx_with_revelcy(&dto.unsigned_tx, ctx.network, extra_signers.as_deref()).map_err(|e| {
        eprintln!("sign_transaction error ({tx_type}): {e:?}");
        ApiError::internal_sign_tx_failed()
    })?;

    let res = send_signed_tx_base64(ctx.network, &signed).await.map_err(|e| {
        eprintln!("send_signed_tx_base64 error ({tx_type}): {e:?}");
        ApiError::internal_send_tx_failed()
    })?;

    wait_for_confirmed(
        ctx.network,
        &res,
        Duration::from_secs(60),
        Duration::from_millis(300), // 0.3 sec
    ).await.map_err(|e| {
        eprintln!("wait_for_confirmed error ({tx_type}): {e:?}");
        ApiError::internal_confirm_tx_failed()
    })?;

    background_finalize_action(
        ctx.network,
        res,
        Duration::from_secs(20*60), // timeout
        Duration::from_secs(1),   // poll_every
        update_method,
    );

    Ok(HttpResponse::Ok().json(SentTxResponse { 
        signature: res.to_string(), 
        status: TransactionStatus::Confirmed }))
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
        user: ctx.user.current_pubkey,
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
    let full = premarket_service::get_full_premarket_info(pool.get_ref(), &dto.premarket_account, PremarketLookupKeyType::BcAddress)
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
        user: ctx.user.current_pubkey,
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

    if premarket_info.main_info.is_hided && key_type == PremarketLookupKeyType::BcAddress {
        let ctx: super::auth_validation::BaseRequestContext = match validate_base_request(&req, query.network.as_str(), None) {
            Ok(v) => v,
            Err(_) => return Err(ApiError::invalid_auth_token()),
        };

        let creator_id_ok = premarket_info.main_info.creator.id == Some(ctx.user.internal_id);

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
    let pm = premarket_service::get_full_premarket_info(&pool, &premarket_pubkey, PremarketLookupKeyType::BcAddress)
        .await
        .map_err(|_| ApiError::internal_update_db_error())?
        .ok_or_else(ApiError::missing_premarket)?;

    // 3) check creator
    if !(pm.main_info.creator.id == Some(ctx.user.internal_id)) {
        return Err(ApiError::forbidden());
    }


    // 4) normalize inputs

    // 5) save
    premarket_service::update_availability_info(
        pool.get_ref(),
        &premarket_pubkey,
        dto.is_hided,
        dto.token_short_url_name,
    )
    .await
    .map_err(|_| ApiError::internal_update_db_error())?;
    println!("Update availability Done");


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
            TokenState::Premarket => PremarketState::Premarket,
            TokenState::Canceled => PremarketState::Canceled,
            TokenState::Finished => PremarketState::Finished,
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