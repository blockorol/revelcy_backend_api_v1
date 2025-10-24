use std::str::FromStr;
use sqlx::{PgPool};
use uuid::Uuid;
use chrono::Utc;
use actix_web::error::ErrorInternalServerError;

use actix_web::{web, Error, HttpResponse, HttpRequest};
use actix_web::HttpMessage;
use solana_sdk::pubkey::Pubkey;

use crate::api::premarket::{
    AddUsersToWhitelistRequest, GetWhitelistRequest,
    JoinPremarketTxRequest, OutPremarketTxRequest, TxOnlyResponse, FinishPremarketTxRequest,
    CreatePremarketTxRequest, CreatePremarketTxResponse,
    GetListQuery, GetListMainInfoDTO, UpdateCommunityDTO,
    FinishedPremarketDTO,
    PremarketTransactionDTO, HolderInfoDTO, UserJoinedToPremarketDTO,
    GetDynamicInfoQuery, TokenDynamicInfoDTO, CreatePremarketDTO, BlockchainInfoDTO,
    CommunityInfoDTO, CommunityLinkDTO, GetMainInfoDTO, GetMainInfoQuery, TokenLinksDTO, TokenState,
    DistributeTokensRequest,
    KillPremarketTxRequest,
    UpdatePremarketDataDTO,
    DeployTxDTO,
    CheckTxDTO,
};
use crate::models::premarket::{
    BuildFinishTxParams, 
    BuildJoinTxParams, 
    BuildKillTxParams, 
    BuildOutTxParams, 
    BuildPremarketTxParams, 
    CommunityInfoServiceModel, 
    CommunityLink, DeployTxParams,
    DistributeTokensParams, GetPremarketDataParams,
    HolderInfo, PremarketGoal, PremarketInfoServiceModel,
    PremarketListResult, PremarketState, SolanaNetwork, TokenInfo, 
    TokenLinks, UpdatePremarketDataParams, UserInfoShort,
    CheckTxParams, 
};

use crate::services::{
    whitelist_service,
    premarket_service,
    jwt_service,
};
use crate::middleware::jwt::JwtMiddleware;

use crate::services::solana_service::{
    build_create_premarket_tx, 
    build_join_premarket_tx, 
    build_out_premarket_tx, 
    build_finish_premarket_tx, 
    build_kill_premarket_tx,
    test_build_kill_premarket_tx,
    distribute_tk,
    get_premarket_data,
    update_premarket_data,
    deploy_tx_service,
    check_tx_service,
};

pub fn pub_scope() -> impl actix_web::dev::HttpServiceFactory {
    web::scope("/premarket")
        .wrap(JwtMiddleware)
        .route("/get_main_info", web::get().to(get_main_info))
        .route("/get_list", web::get().to(get_list_main_info))
        .route("/get_dynamic_info", web::get().to(get_dynamic_info))
        .route("/created", web::post().to(created_premarket))
        .route("/update_community", web::post().to(update_community_info))
        .route("/user_joined", web::post().to(user_joined))
        .route("/user_out", web::post().to(user_out))
        .route("/finished", web::post().to(finished_premarket))
        .route("/killed", web::post().to(killed_premarket))
        .route("/distribute_tokens", web::post().to(distribute_tokens))
        .route("/update", web::post().to(update_premarket_data_tx))
        
        .route("/whitelist/add", web::post().to(add_users_to_whitelist))
        .route("/whitelist", web::post().to(get_whitelist))

        .route("/deploy_tx",   web::post().to(deploy_tx))
        .route("/check_tx",   web::post().to(check_tx))

        .route("/tx/create", web::post().to(create_premarket_tx))
        .route("/tx/join",   web::post().to(join_premarket_tx))
        .route("/tx/out",    web::post().to(out_premarket_tx))
        .route("/tx/finish", web::post().to(finish_premarket_tx))
        .route("/tx/kill",   web::post().to(kill_premarket_tx))

        .route("/tx/test_kill",   web::post().to(test_kill_premarket_tx))
}

pub async fn get_whitelist(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    payload: web::Json<GetWhitelistRequest>,
) -> Result<HttpResponse, Error> {
    let dto = payload.into_inner();

    // 1) auth
    let token = req
        .extensions()
        .get::<String>()
        .cloned()
        .ok_or_else(|| ErrorUnauthorized("missing auth token"))?;
    let token_data = jwt_service::decode_jwt_with_user_info(&token)
        .map_err(|_| ErrorUnauthorized("invalid token"))?;
    let caller_user_id = token_data
        .user_id
        .ok_or_else(|| ErrorUnauthorized("no user_id in token"))?;

    // 2) resolve premarket_id (DB UUID)
    let premarket_db_id: Uuid = if let Some(id) = dto.premarket_id {
        id
    } else if let Some(pubkey_str) = dto.premarket_pubkey.clone() {
        let _ = Pubkey::from_str(&pubkey_str)
            .map_err(|_| ErrorBadRequest("invalid premarket_pubkey"))?;
        let info = premarket_service::get_full_premarket_info(pool.get_ref(), &pubkey_str)
            .await
            .map_err(ErrorInternalServerError)?
            .ok_or_else(|| ErrorBadRequest("premarket not found"))?;
        info.main_info
            .id
            .ok_or_else(|| ErrorInternalServerError("premarket exists but has no DB id"))?
    } else {
        return Err(ErrorBadRequest(
            "provide either premarket_id (UUID) or premarket_pubkey",
        ));
    };

    // 3) owner check (только создатель может читать вайтлист)
    let pm_full = premarket_service::get_full_premarket_info_by_id(pool.get_ref(), premarket_db_id)
        .await
        .map_err(ErrorInternalServerError)?;
    let owner_id = pm_full
        .as_ref()
        .and_then(|i| i.main_info.creator.id)
        .ok_or_else(|| ErrorBadRequest("premarket not found or has no owner"))?;

    if owner_id != caller_user_id {
        return Err(ErrorForbidden("only premarket owner can read whitelist"));
    }

    // 4) получаем подробную инфу о пользователях из whitelist
    let users = user_service::get_users_info(pool.clone(), 
        // сначала достаём id'шники, затем user_service агрегирует инфу
        // (если ты уже сделал whitelist_service::get_whitelist_users — можно звать его напрямую)
        crate::storage::whitelist_repo::get_whitelist(pool.get_ref(), premarket_db_id)
            .await
            .map_err(ErrorInternalServerError)?,
    )
    .await
    .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(users))
}


pub async fn add_users_to_whitelist(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    payload: web::Json<AddUsersToWhitelistRequest>,
) -> Result<HttpResponse, Error> {
    let dto = payload.into_inner();

    let token = req
        .extensions()
        .get::<String>()
        .cloned()
        .ok_or_else(|| ErrorUnauthorized("missing auth token"))?;
    let token_data = jwt_service::decode_jwt_with_user_info(&token)
        .map_err(|_| ErrorUnauthorized("invalid token"))?;

    let current_user_wallet = token_data
        .current_wallet
        .ok_or_else(|| ErrorUnauthorized("no user_id in token"))?;

    
    let premarket_data = if let Some(id) = dto.premarket_id {
        let info: crate::models::premarket::FullPremarketInfo = premarket_service::get_premarket_info_by_bc_address(pool.get_ref(), id)
        .await
        .map_err(ErrorInternalServerError)?
        .ok_or_else(|| ErrorBadRequest("premarket not found"))?;

        info
    } else if let Some(pubkey_str) = dto.premarket_pubkey.clone() {
        let pk = Pubkey::from_str(&pubkey_str)
            .map_err(|_| ErrorBadRequest("invalid premarket_pubkey"))?;
        let info: crate::models::premarket::FullPremarketInfo = premarket_service::get_full_premarket_info(pool.get_ref(), &pk.to_string())
            .await
            .map_err(ErrorInternalServerError)?
            .ok_or_else(|| ErrorBadRequest("premarket not found"))?;
        info
    } else {
        return Err(ErrorBadRequest(
            "provide either premarket_id (UUID) or premarket_pubkey",
        ));
    };

    // let owner_check = 
    let owner_id = owner_check
        .as_ref()
        .and_then(|i| i.main_info.creator.id)
        .ok_or_else(|| ErrorBadRequest("premarket not found or has no owner"))?;

    if premarket_data.main_info.creator.blockchain_address != current_user_wallet {
        return Err(ErrorForbidden("only premarket owner can modify whitelist"));
    }

    let mut total_inserted: u64 = 0;
    let premarket_db_id = premarket_data.main_info.id;

    if let Some(user_ids) = dto.user_ids.as_ref() {
        if !user_ids.is_empty() {
            let cnt = whitelist_service::add_user_ids(pool.get_ref(), premarket_db_id, user_ids)
                .await
                .map_err(ErrorInternalServerError)?;
            total_inserted += cnt;
        }
    }

    if let Some(wallets) = dto.wallet_addresses.as_ref() {
        if !wallets.is_empty() {
            let cnt = whitelist_service::add_wallets(pool.get_ref(), premarket_db_id, wallets)
                .await
                .map_err(ErrorInternalServerError)?;
            total_inserted += cnt;
        }
    }

    if total_inserted == 0
        && dto
            .user_ids
            .as_ref()
            .map(|v| v.is_empty())
            .unwrap_or(true)
        && dto
            .wallet_addresses
            .as_ref()
            .map(|v| v.is_empty())
            .unwrap_or(true)
    {
        return Err(ErrorBadRequest(
            "provide at least one of: non-empty user_ids or wallet_addresses",
        ));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "premarket_id": premarket_db_id,
        "inserted": total_inserted
    })))
}


pub async fn create_premarket_tx(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    payload: web::Json<CreatePremarketTxRequest>,
) -> Result<HttpResponse, Error> {
    let dto = payload.into_inner();
    let _token = req.extensions().get::<String>().cloned();

    let token_data = jwt_service::decode_jwt_with_user_info(&_token.unwrap_or_default()).unwrap();

    let network = match SolanaNetwork::try_from(dto.network.as_str()) {
        Ok(n) => n,
        Err(_) => return Ok(HttpResponse::BadRequest().body("invalid network")),
    };

    let user = match Pubkey::from_str(&dto.user_pubkey) {
        Ok(p) => p,
        Err(_) => return Ok(HttpResponse::BadRequest().body("invalid user_pubkey")),
    };

    match token_data.current_wallet {
        Some(pk) => {
            if pk != dto.user_pubkey {
                println!("user_pubkey does not match token");
                return Ok(HttpResponse::Forbidden().body("user_pubkey does not match token"));
            } else {
                println!("user_pubkey matches token");
            }
        }
        None => return Ok(HttpResponse::Unauthorized().body("user_pubkey does not match token")),
    }

    let now = Utc::now().timestamp();
    if dto.deadline < now + 60 * 60 - 1 {
        return Ok(HttpResponse::BadRequest().body("deadline must be at least +1h from now"));
    }

    if dto.goal_sol_lamp == 0 || dto.max_sol_lamp == 0 || dto.max_sol_lamp < dto.goal_sol_lamp {
        return Ok(HttpResponse::BadRequest().body("invalid goal/max values"));
    }

    let params = BuildPremarketTxParams {
        network,
        user,
        name: dto.name,
        symbol: dto.symbol,
        uri: dto.uri,
        deadline: dto.deadline,
        goal: dto.goal_sol_lamp,
        max: dto.max_sol_lamp,
        creator_allocate: dto.creator_allocate_lamp,
    };

    match build_create_premarket_tx(pool.get_ref(), params).await {
        Ok(res) => {
            let body = CreatePremarketTxResponse {
                transaction: res.tx_base64,
                premarket_account_pda: res.premarket_pda.to_string(),
                mint_address: res.mint_address.clone(),
            };
            Ok(HttpResponse::Ok().json(body))
        }
        Err(e) => {
            eprintln!("build_create_premarket_tx error: {e:?}");
            Ok(HttpResponse::InternalServerError().body("failed to build transaction"))
        }
    }
}

pub async fn join_premarket_tx(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    payload: web::Json<JoinPremarketTxRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let dto = payload.into_inner();

    let network = match SolanaNetwork::try_from(dto.network.as_str()) {
        Ok(n) => n,
        Err(_) => return Ok(HttpResponse::BadRequest().body("invalid network")),
    };
    let user = match Pubkey::from_str(&dto.user_pubkey) {
        Ok(p) => p,
        Err(_) => return Ok(HttpResponse::BadRequest().body("invalid user_pubkey")),
    };

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

    let premarket = match Pubkey::from_str(&dto.premarket_account) {
        Ok(p) => p,
        Err(_) => return Ok(HttpResponse::BadRequest().body("invalid premarket_account")),
    };
    let premarket_data = match premarket_service::get_full_premarket_info(&pool, &pubkey.to_string()).await {
        Ok(Some(info)) => info,
        Ok(None) => return HttpResponse::NotFound().body("Premarket info not found"),
        Err(err) => {
            eprintln!("Error fetching premarket info: {:?}", err);
            return HttpResponse::InternalServerError().finish();
        }
    };
    if (premarket_data.main_info.finished_timestamp) {
        return Ok(HttpResponse::BadRequest().body("Premarket already finished"));
    }
    match whitelist_service::has_user_access(pool, premarket_data.main_info.id, token_data.user_id).await {
        Ok(res) => {
            if !res {
                return HttpResponse::Forbidden().body("You are not in whitelist");
            }
        },
        Err(err) => {
            eprintln!("Error fetching premarket info: {:?}", err);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // todo: add this logic - don't show to iser
    // if (premarket_data.main_info.deadline_timestamp < ) {
    //     return Ok(HttpResponse::BadRequest().body("Premarket already finished"));
    // }

    if dto.amount_sol_lamp == 0 {
        return Ok(HttpResponse::BadRequest().body("amount_sol_lamp must be > 0"));
    }

    let params = BuildJoinTxParams {
        network,
        user,
        premarket,
        amount: dto.amount_sol_lamp,
    };

    match build_join_premarket_tx(params).await {
        Ok(res) => Ok(HttpResponse::Ok().json(TxOnlyResponse { transaction: res.tx_base64 })),
        Err(e) => {
            eprintln!("build_join_premarket_tx error: {e:?}");
            Ok(HttpResponse::InternalServerError.body(format!("error: {e}")))
        }
    }
}

pub async fn out_premarket_tx(
    req: HttpRequest,
    payload: web::Json<OutPremarketTxRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let dto = payload.into_inner();

    let network = match SolanaNetwork::try_from(dto.network.as_str()) {
        Ok(n) => n,
        Err(_) => return Ok(HttpResponse::BadRequest().body("invalid network")),
    };
    let user = match Pubkey::from_str(&dto.user_pubkey) {
        Ok(p) => p,
        Err(_) => return Ok(HttpResponse::BadRequest().body("invalid user_pubkey")),
    };

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

    let premarket = match Pubkey::from_str(&dto.premarket_account) {
        Ok(p) => p,
        Err(_) => return Ok(HttpResponse::BadRequest().body("invalid premarket_account")),
    };

    let params = BuildOutTxParams { network, user, premarket };

    match build_out_premarket_tx(params).await {
        Ok(res) => Ok(HttpResponse::Ok().json(TxOnlyResponse { transaction: res.tx_base64 })),
        Err(e) => {
            eprintln!("build_out_premarket_tx error: {e:?}");
            Ok(HttpResponse::BadRequest().body(format!("error: {e}")))
        }
    }
}

pub async fn finish_premarket_tx(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    payload: web::Json<FinishPremarketTxRequest>,
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

    let params = BuildFinishTxParams { network, user, premarket };

    match build_finish_premarket_tx(pool.get_ref(), params).await {
        Ok(res) => Ok(HttpResponse::Ok().json(TxOnlyResponse { transaction: res.tx_base64 })),
        Err(e) => {
            eprintln!("build_finish_premarket_tx error: {e:?}");
            Ok(HttpResponse::InternalServerError().body("failed to build finish tx"))
        }
    }
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

    match build_kill_premarket_tx(pool.get_ref(), params).await {
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
            premarket_goal_pers: premarket_info.goal.percent,
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
        premarket_goal_pers: premarket_info.main_info.goal.percent,
        premarket_goal_sol_lamp: premarket_info.main_info.goal.solana_lamp.to_string(),
        premarket_deadline: premarket_info.main_info.deadline_timestamp,
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


    let premarket = CreatePremarketInfoServiceModel {
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
            percent: info.premarket_goal_pers,
            solana_lamp: solana_lamp,
        },
        deadline_timestamp: info.premarket_deadline,
        created_timestamp: info.premarket_created,
        blockchain_address: info.premarket_address,
        finished_timestamp: None
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
    


    let network_str = dto.network;
    let network = SolanaNetwork::try_from(network_str.as_str())
        .map_err(|_| actix_web::error::ErrorBadRequest("invalid network"))?;
    let premarket = Pubkey::from_str(&dto.base.premarket_pub_key)
        .map_err(|_| actix_web::error::ErrorBadRequest("invalid premarket pubkey"))?;
    let user = Pubkey::from_str(&dto.base.user_wallet)
        .map_err(|_| actix_web::error::ErrorBadRequest("invalid user wallet"))?;
    let params = GetPremarketDataParams { network, premarket };
    let premarket_data = get_premarket_data(params).await?;
    let users = premarket_data.users
        .into_iter()
        .map(|u| u.wallet.to_string())
        .collect();
    let params = DistributeTokensParams {
        network: network,
        user: user,
        premarket: premarket,
        token_mint: premarket_data.mint,
        users: users,
    };
    match distribute_tk(&pool, params).await {
        Ok(_) => {
            let msg = format!("Tokens distributed!");
            println!("{}", msg);
        }
        Err(e) => {
            eprintln!("Error distributing tokens: {:?}", e);
            eprintln!("Manual distribution needed for mint: {:?}", &premarket_data.mint);
        }
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
    pool: web::Data<sqlx::PgPool>,
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

    match update_premarket_data(pool.get_ref(), params).await {
        Ok(res) => Ok(HttpResponse::Ok().json(TxOnlyResponse { transaction: res.tx_base64 })),
        Err(e) => {
            eprintln!("build_kill_premarket_tx error: {e:?}");
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