use crate::models::premarket::{
    PremarketLookupKeyType,
    CommunityInfoServiceModel, CommunityLink, FullPremarketInfo, HolderInfo,
    JoinConfirmationStatusDTO, LinkType, OutConfirmationStatusDTO, PremarketGoal,
    PremarketInfoServiceModel, PremarketListResult, PremarketOnchainData, PremarketOnchainUser,
    PremarketState, TokenDynamicInfo, TokenInfo, TokenLinks, TxConfirmationStatusDTO,
    UserInfoShort,
};
use crate::models::user::UserContextData;

use crate::storage::models::{
    CommunityInfoDbModel, CommunityLinkDbModel, HolderDbModel, PremarketInfoDbModel
}; // todo: по хорошему убрать это. сервисный уровень не должен знать про модели БД. он рабоатет с моделями сервиса, и каст должен идти в репозитории
use crate::storage::premarket_repo;
use actix_web::error::ErrorBadRequest;
use actix_web::error::ErrorInternalServerError;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::premarket::PythResponse;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

pub async fn get_full_premarket_info(
    pool: &PgPool,
    key: &str,
    key_type: PremarketLookupKeyType,
) -> Result<Option<FullPremarketInfo>, actix_web::Error> {
    let data = match key_type {
        PremarketLookupKeyType::BcAddress => {
            premarket_repo::get_premarket_info_by_bc_address(pool, key).await
        }
        PremarketLookupKeyType::Name => {
            premarket_repo::get_premarket_info_by_name(pool, key).await
        }
    };

    match data {
        Ok(Some((pm_db, cm_db, links_db))) => {
            let premarket_info = PremarketInfoServiceModel {
                id: Some(pm_db.id),
                blockchain_address: pm_db.bc_address,
                short_url_name: pm_db.short_url_name,
                creator: UserInfoShort {
                    id: Some(pm_db.creator_id),
                    blockchain_address: pm_db.creator_address,
                },
                token_info: TokenInfo {
                    address: pm_db.mint_address,
                    name: pm_db.name,
                    description: pm_db.description,
                    symbol: pm_db.symbol,
                    image_url: pm_db.image_url,
                    data_uri: pm_db.data_uri,
                    links: TokenLinks {
                        telegram: pm_db.telegram,
                        twitter: pm_db.twitter,
                        web_site: pm_db.web_site,
                    },
                },
                state: pm_db.state.into(),
                goal: PremarketGoal {
                    solana_lamp: pm_db.premarket_goal_sol_lamp,
                },
                is_extended: pm_db.is_extended,
                is_hided: pm_db.is_hided,
                deadline_timestamp: pm_db.premarket_deadline,
                created_timestamp: pm_db.premarket_created,
                finished_timestamp: pm_db.premarket_finished,
            };

            let links = if links_db.is_empty() {
                None
            } else {
                Some(
                    links_db
                        .into_iter()
                        .map(|l| CommunityLink {
                            text: l.text,
                            url: l.url,
                            r#type: match l.r#type.as_str() {
                                "x" => LinkType::X,
                                "tg" => LinkType::Tg,
                                _ => LinkType::Other,
                            },
                        })
                        .collect(),
                )
            };

            let community_info = CommunityInfoServiceModel {
                description: cm_db.description,
                token_banner_url: cm_db.token_banner_url,
                links,
            };

            Ok(Some(FullPremarketInfo {
                main_info: premarket_info,
                community: community_info,
            }))
        }
        Ok(None) => Ok(None),
        Err(e) => Err(ErrorInternalServerError(e)),
    }
}

pub async fn get_list(
    pool: &PgPool,
    cursor: i64,
    limit: i64,
    user_opt: Option<UserContextData>
) -> Result<Option<PremarketListResult>, actix_web::Error> {
    let user_id_opt = user_opt.as_ref().map(|u| u.internal_id);

    let res = premarket_repo::get_list(pool, user_id_opt, cursor, limit)
        .await
        .map_err(ErrorInternalServerError)?;

    let (rows, total) = match res {
        Some((rows, total)) => (rows, total),
        None => {
            return Ok(Some(PremarketListResult {
                items: Vec::new(),
                total: Some(0),
            }));
        }
    };

    let items = rows
        .into_iter()
        .map(|pm_db: PremarketInfoDbModel | PremarketInfoServiceModel {
            id: Some(pm_db.id),
            blockchain_address: pm_db.bc_address,
            short_url_name: pm_db.short_url_name,
            creator: UserInfoShort {
                id: Some(pm_db.creator_id),
                blockchain_address: pm_db.creator_address,
            },
            token_info: TokenInfo {
                address: pm_db.mint_address,
                name: pm_db.name,
                description: pm_db.description,
                symbol: pm_db.symbol,
                image_url: pm_db.image_url,
                data_uri: pm_db.data_uri,
                links: TokenLinks {
                    telegram: pm_db.telegram,
                    twitter: pm_db.twitter,
                    web_site: pm_db.web_site,
                },
            },
            state: pm_db.state.into(),
            goal: PremarketGoal {
                solana_lamp: pm_db.premarket_goal_sol_lamp,
            },
            is_extended: pm_db.is_extended,
            is_hided: pm_db.is_hided,
            deadline_timestamp: pm_db.premarket_deadline,
            created_timestamp: pm_db.premarket_created,
            finished_timestamp: pm_db.premarket_finished,
        })
        .collect();

    Ok(Some(PremarketListResult {
        items,
        total: Some(total),
    }))
}

pub async fn create_full_premarket_info(
    pool: &PgPool,
    premarket: PremarketInfoServiceModel,
    community: CommunityInfoServiceModel,
) -> Result<(), actix_web::Error> {
    let premarket_db: PremarketInfoDbModel = PremarketInfoDbModel {
        id: uuid::Uuid::new_v4(),
        bc_address: premarket.blockchain_address,
        short_url_name: premarket.short_url_name,
        creator_address: premarket.creator.blockchain_address,
        creator_id: premarket.creator.id.unwrap_or(Uuid::nil()),
        mint_address: premarket.token_info.address,
        name: premarket.token_info.name,
        description: premarket.token_info.description,
        symbol: premarket.token_info.symbol,
        image_url: premarket.token_info.image_url,
        data_uri: premarket.token_info.data_uri,
        telegram: premarket.token_info.links.telegram,
        twitter: premarket.token_info.links.twitter,
        web_site: premarket.token_info.links.web_site,
        premarket_goal_sol_lamp: premarket.goal.solana_lamp,
        premarket_deadline: premarket.deadline_timestamp,
        premarket_created: premarket.created_timestamp,
        premarket_finished: premarket.finished_timestamp,
        is_extended: false,
        is_hided: premarket.is_hided,
        state: premarket.state.to_string(),
    };

    let community_db = CommunityInfoDbModel {
        id: premarket_db.id,
        description: community.description,
        token_banner_url: community.token_banner_url,
    };

    let link_db = community.links.map(|links_vec| {
        links_vec
            .into_iter()
            .map(|l: CommunityLink| CommunityLinkDbModel {
                id: uuid::Uuid::new_v4(),
                community_info_id: Uuid::nil(),
                text: l.text,
                url: l.url,
                r#type: match l.r#type {
                    LinkType::X => "x".to_string(),
                    LinkType::Tg => "tg".to_string(),
                    _ => "other".to_string(),
                },
            })
            .collect::<Vec<_>>()
    });

    premarket_repo::create_premarket_and_community(pool, &premarket_db, &community_db, link_db)
        .await
        .map_err(ErrorInternalServerError)
}

pub async fn update_availability_info(
    pool: &PgPool,
    premarket_pubkey: &str,
    is_hided: Option<bool>,
    short_url_name: Option<String>,
) -> Result<(), actix_web::Error> {
    premarket_repo::update_availability_info(
        pool,
        premarket_pubkey,
        is_hided,
        short_url_name,
    )
    .await
    .map_err(ErrorInternalServerError)
}

pub async fn update_community_info(
    pool: &PgPool,
    bc_address: &str,
    community: CommunityInfoServiceModel,
) -> Result<(), actix_web::Error> {
    let link_db = community.links.map(|links_vec| {
        links_vec
            .into_iter()
            .map(|l: CommunityLink| CommunityLinkDbModel {
                id: uuid::Uuid::new_v4(),
                community_info_id: Uuid::nil(),
                text: l.text,
                url: l.url,
                r#type: match l.r#type {
                    LinkType::X => "x".to_string(),
                    LinkType::Tg => "tg".to_string(),
                    _ => "other".to_string(),
                },
            })
            .collect::<Vec<_>>()
    });

    premarket_repo::update_community_info(
        pool,
        bc_address, // было &bc_address
        &community.description,
        community.token_banner_url.as_deref(), // Option<String> -> Option<&str>
        link_db,                               // было &link_db
    )
    .await
    .map_err(ErrorInternalServerError)
}

pub async fn get_dynamic_info(
    pool: &PgPool,
    premarket_pubkey: &str,
) -> Result<TokenDynamicInfo, actix_web::Error> {
    let holder_limit = 300;
    let holder_data =
        premarket_repo::get_holders_by_premarket_address(pool, premarket_pubkey, holder_limit)
            .await
            .map_err(ErrorInternalServerError)?;

    let holder_service_list: Vec<HolderInfo> = holder_data
        .holders
        .into_iter()
        .map(|h| HolderInfo {
            id: h.holder_id,
            wallet_address: h.holder_wallet,
            join_timestamp: h.join_timestamp,
            icon_url: h.avatar_url,
            username: h.username,
            amount_sol_lamp: h.amount_lamport as u64,
            claimed: h.claimed,
        })
        .collect();

    let current_price_lamp = get_price_by_market_cap(holder_data.reserved_sol_lamp as u64).await;

    let price_24h_ago_lamp =
        get_price_by_market_cap(holder_data.reserved_sol_24h_before_lamp as u64).await;

    let change_24h = if price_24h_ago_lamp > 0.0 && price_24h_ago_lamp != current_price_lamp {
        ((current_price_lamp - price_24h_ago_lamp) / price_24h_ago_lamp) * 100.0
    } else if price_24h_ago_lamp == current_price_lamp {
        0.01
    } else {
        0.0
    };

    Ok(TokenDynamicInfo {
        holders_count: holder_data.total_active_count as u32,
        current_price_lamp,
        reserved_sol_lamp: holder_data.reserved_sol_lamp as u64,
        change_24h,
        holders: holder_service_list,
    })
}

pub async fn set_premarket_state(
    pool: &PgPool,
    premarket_pubkey: &str,
    new_state: PremarketState,
    premarket_finished: Option<i64>,
) -> Result<(), actix_web::Error> {
    let affected = premarket_repo::update_premarket_state(
        pool,
        premarket_pubkey,
        &new_state.to_string(),
        premarket_finished,
    )
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    if affected == 0 {
        return Err(actix_web::error::ErrorNotFound(format!(
            "premarket '{}' not found",
            premarket_pubkey
        )));
    }

    Ok(())
}

pub async fn add_holder(
    pool: &PgPool,
    premarket_pubkey: &str,
    holder: HolderInfo,
) -> Result<(), actix_web::Error> {
    let holder = HolderDbModel {
        id: uuid::Uuid::new_v4(),
        premarket_info_id: Uuid::nil(),
        holder_wallet: holder.wallet_address,
        holder_id: holder.id,
        join_timestamp: holder.join_timestamp,
        amount_lamport: holder.amount_sol_lamp as i64,
        out_timestamp: None,
        claimed: false,
        avatar_url: None,
        username: None,
    };
    premarket_repo::insert_holder(pool, premarket_pubkey, &holder)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)
}

pub async fn remove_holder(
    pool: &PgPool,
    premarket_pubkey: &str,
    wallet_address: &str,
) -> Result<(), actix_web::Error> {
    premarket_repo::soft_delete_holder(
        pool,
        premarket_pubkey,
        wallet_address,
        Utc::now().timestamp(),
    )
    .await
    .map(|_| ()) // игнорируем u64, возвращаем ()
    .map_err(actix_web::error::ErrorInternalServerError)
}

pub async fn get_price_by_market_cap(real_lamp_amount: u64) -> f64 {
    let url = match std::env::var("PYTH_MAINNET_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("PYTH_MAINNET_URL environment variable not set");
            return 0.0;
        }
    };

    let current_sol_price = match reqwest::get(&url).await {
        Ok(response) => {
            match response.json::<PythResponse>().await {
                Ok(pyth_response) => {
                    //println!("Pyth API response: {:?}", pyth_response);
                    if let Some(parsed_data) = pyth_response.parsed.first() {
                        // Parse the price string and apply the exponent
                        let price_str = &parsed_data.price.price;
                        let expo = parsed_data.price.expo;
                        let price_value: f64 = price_str.parse().unwrap_or(0.0);
                        let price = price_value * 10_f64.powi(expo);
                        price
                    } else {
                        println!("No parsed data found in Pyth response");
                        0.0
                    }
                }
                Err(e) => {
                    println!("Pyth JSON parsing error: {:?}", e);
                    0.0
                }
            }
        }
        Err(e) => {
            println!("Pyth HTTP request error: {:?}", e);
            0.0
        }
    };

    let real_sol_amount: f64 = real_lamp_amount as f64 / 1_000_000_000.0;

    let real_token_bought_amount: u64 =
        ((1_073_000_000.0 * real_sol_amount) / (30.0 + real_sol_amount)) as u64;
    println!("Real token bought amount: {}", real_token_bought_amount);
    let real_token_amount: u64 = 793_100_000 - real_token_bought_amount;

    let virtual_lamp_amount: f64 = real_sol_amount + 30.0;
    let virtual_token_amount = real_token_amount + 279_900_000;

    let price = virtual_lamp_amount / virtual_token_amount as f64 * current_sol_price;
    let final_price = (price * 1_000_000_000.0).round() / 1_000_000_000.0;

    println!("Real sol: {}, token: {}, Virtual sol: {}, token: {} => price {}, Final price: {}", real_sol_amount, real_token_amount, virtual_lamp_amount, virtual_token_amount, price, final_price  );
    final_price
}

pub async fn get_tx_confirmation_status(
    client: &RpcClient,
    tx_id: &str,
) -> Result<TxConfirmationStatusDTO, actix_web::Error> {
    let tx_signature = match solana_sdk::signature::Signature::from_str(tx_id) {
        Ok(sig) => sig,
        Err(_) => {
            return Err(actix_web::error::ErrorBadRequest(
                "Invalid transaction signature format",
            ))
        }
    };

    let status = client
        .get_signature_status(&tx_signature)
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!(
                "Failed to get transaction status: {}",
                e
            ))
        })?;

    match status {
        Some(status) if status.is_ok() => Ok(TxConfirmationStatusDTO::Confirmed),
        Some(_) => Ok(TxConfirmationStatusDTO::Failed),
        None => Ok(TxConfirmationStatusDTO::Pending),
    }
}

pub async fn check_user_joined(
    client: &RpcClient,
    user: &str,
    premarket_account: &str,
) -> Result<JoinConfirmationStatusDTO, actix_web::Error> {
    let user_pubkey = Pubkey::from_str(user)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid user public key"))?;
    let premarket_pubkey = Pubkey::from_str(premarket_account)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid premarket account public key"))?;

    let is_holder = get_premarket_data(client, &premarket_pubkey)
        .await
        .map(|data| data.users.iter().any(|user| user.wallet == user_pubkey))
        .unwrap_or(false);

    Ok(if is_holder {
        JoinConfirmationStatusDTO::JoinSuccess
    } else {
        JoinConfirmationStatusDTO::JoinFailed
    })
}

pub async fn check_user_out(
    client: &RpcClient,
    user: &str,
    premarket_account: &str,
) -> Result<OutConfirmationStatusDTO, actix_web::Error> {
    let user_pubkey = Pubkey::from_str(user)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid user public key"))?;
    let premarket_pubkey = Pubkey::from_str(premarket_account)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid premarket account public key"))?;

    let is_holder = get_premarket_data(client, &premarket_pubkey)
        .await
        .map(|data| data.users.iter().any(|user| user.wallet == user_pubkey))
        .unwrap_or(false);

    Ok(if is_holder {
        OutConfirmationStatusDTO::OutFailed
    } else {
        OutConfirmationStatusDTO::OutSuccess
    })
}

pub async fn get_premarket_data(
    client: &RpcClient,
    premarket_account: &Pubkey,
) -> Result<PremarketOnchainData, actix_web::Error> {
    let account = client
        .get_account(premarket_account)
        .await
        .map_err(|e| ErrorInternalServerError(format!("Failed to fetch account: {e}")))?;

    // Minimum length: 8 discriminator + 4 length + (could be zero users) + 8+1+8+8+32 for tail fields
    if account.data.len() < 8 + 4 + 8 + 1 + 8 + 8 + 32 {
        return Err(ErrorBadRequest(
            "Account data too short for premarket layout",
        ));
    }

    // Skip discriminator
    let data = &account.data[8..];

    // Read users vector length
    let users_len_bytes: [u8; 4] = data[0..4].try_into().unwrap();
    let users_len = u32::from_le_bytes(users_len_bytes) as usize;

    // Each user entry: 32 (pubkey) + 8 (lamports) + 1 (claimed)
    let users_section_len = users_len
        .checked_mul(41)
        .ok_or_else(|| ErrorBadRequest("Users length overflow"))?;

    let needed_len = 4 + users_section_len + (8 + 1 + 8 + 8 + 32); // vec length + users + tail fields (end_timestamp + extended_premarket + goal + max + mint)
    if data.len() < needed_len {
        return Err(ErrorBadRequest(
            "Account data too short for declared users length",
        ));
    }

    let mut users = Vec::with_capacity(users_len);
    let mut offset = 4;
    for _ in 0..users_len {
        let pk_slice = &data[offset..offset + 32];
        let wallet = Pubkey::new_from_array(pk_slice.try_into().unwrap());
        let lamports = u64::from_le_bytes(data[offset + 32..offset + 40].try_into().unwrap());
        let claimed = data[offset + 40] != 0;
        users.push(PremarketOnchainUser {
            wallet,
            contributed_lamports: lamports,
            claimed,
        });
        offset += 41;
    }

    // Tail fields
    let end_timestamp = i64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
    offset += 8;

    let extended_premarket = data[offset] != 0;
    offset += 1;

    let goal_lamports = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
    offset += 8;

    let max_lamports = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
    offset += 8;

    let mint = Pubkey::new_from_array(data[offset..offset + 32].try_into().unwrap());

    Ok(PremarketOnchainData {
        users,
        end_timestamp,
        extended_premarket,
        goal_lamports,
        max_lamports,
        mint,
    })
}

pub async fn get_holder_entry_price(
    pool: &PgPool,
    premarket_pubkey: &str,
    holder_wallet: &str,
) -> Result<Option<f64>, actix_web::Error> {
    // Get the holder's join timestamp
    let join_timestamp = match premarket_repo::get_holder_join_timestamp(
        pool,
        premarket_pubkey,
        holder_wallet,
    )
    .await
    {
        Ok(Some(ts)) => ts,
        Ok(None) => return Ok(None),
        Err(e) => return Err(ErrorInternalServerError(e)),
    };

    // Get the total lamports collected before this holder joined
    let lamports_before_join =
        premarket_repo::get_lamports_before_timestamp(pool, premarket_pubkey, join_timestamp)
            .await
            .map_err(ErrorInternalServerError)?;

    // Use the same price calculation as get_price_by_market_cap
    let url = match std::env::var("PYTH_MAINNET_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("PYTH_MAINNET_URL environment variable not set");
            return Ok(Some(0.0));
        }
    };

    let current_sol_price = match reqwest::get(&url).await {
        Ok(response) => {
            println!("Pyth API response status: {}", response.status());
            match response.json::<PythResponse>().await {
                Ok(pyth_response) => {
                    if let Some(parsed_data) = pyth_response.parsed.first() {
                        let price_str = &parsed_data.price.price;
                        let expo = parsed_data.price.expo;
                        let price_value: f64 = price_str.parse().unwrap_or(0.0);
                        let price = price_value * 10_f64.powi(expo);
                        price
                    } else {
                        println!("No parsed data found in Pyth response");
                        0.0
                    }
                }
                Err(e) => {
                    println!("Pyth JSON parsing error: {:?}", e);
                    0.0
                }
            }
        }
        Err(e) => {
            println!("Pyth HTTP request error: {:?}", e);
            0.0
        }
    };

    let real_lamp_amount = lamports_before_join as u64;
    let real_sol_amount: f64 = real_lamp_amount as f64 / 1_000_000_000.0;

    let real_token_bought_amount: u64 =
        ((1_073_000_000.0 * real_sol_amount) / (30.0 + real_sol_amount)) as u64;
    let real_token_amount: u64 = 793_100_000 - real_token_bought_amount;

    let virtual_lamp_amount: f64 = real_sol_amount + 30.0;
    let virtual_token_amount = real_token_amount + 279_900_000;

    let price = virtual_lamp_amount / virtual_token_amount as f64 * current_sol_price;
    let final_price = (price * 1_000_000_000.0).round() / 1_000_000_000.0;
    println!("Entry price: {}", final_price);

    Ok(Some(final_price))
}

pub async fn update_premarket_deadline(
    pool: &PgPool,
    premarket_pubkey: &str,
    new_deadline: i64,
) -> Result<(), actix_web::Error> {
    let affected = premarket_repo::update_premarket_deadline(pool, premarket_pubkey, new_deadline)
        .await
        .map_err(ErrorInternalServerError)?;

    if affected == 0 {
        return Err(actix_web::error::ErrorNotFound(format!(
            "premarket '{}' not found",
            premarket_pubkey
        )));
    }

    Ok(())
}

pub async fn update_premarket_links(
    pool: &PgPool,
    premarket_pubkey: &str,
    image_url: Option<String>,
    data_uri: String,
    telegram: Option<String>,
    twitter: Option<String>,
    web_site: Option<String>,
) -> Result<(), actix_web::Error> {
    let payload = premarket_repo::UpdateLinks {
        image_url:image_url,
        data_uri: data_uri,
        telegram: telegram,
        twitter: twitter,
        web_site: web_site,
    };
    let affected = premarket_repo::update_all_links_premarket(pool, premarket_pubkey, payload)
        .await
        .map_err(ErrorInternalServerError)?;

    if affected == 0 {
        return Err(actix_web::error::ErrorNotFound(format!(
            "premarket '{}' not found",
            premarket_pubkey
        )));
    }

    Ok(())
}

pub async fn user_claimed_token(
    pool: &PgPool,
    premarket_pubkey: &Pubkey,
    user_wallet: &str,
) -> Result<(), actix_web::Error> {
    let affected = premarket_repo::update_holder_claimed_status(
                pool,
                &premarket_pubkey.to_string(),
                user_wallet,
                true,
            )
            .await
            .map_err(ErrorInternalServerError)?;
    
    if affected == 0 {
        return Err(actix_web::error::ErrorNotFound(format!(
            "premarket '{}' or holder '{}' not found",
            premarket_pubkey, user_wallet
        )));
    }

    Ok(())
}

pub async fn check_and_update_claimed_status(
    pool: &PgPool,
    client: &RpcClient,
    premarket_pubkey: &Pubkey,
    user_wallet: &str,
) -> Result<(bool, bool), actix_web::Error> {
    // Get on-chain premarket data
    let onchain_data = get_premarket_data(client, premarket_pubkey)
        .await
        .map_err(ErrorInternalServerError)?;

    // Parse user wallet pubkey
    let user_pubkey = Pubkey::from_str(user_wallet)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid user wallet address"))?;

    // Find user in on-chain data
    let user_entry = onchain_data.users.iter().find(|u| u.wallet == user_pubkey);

    match user_entry {
        Some(user) if user.claimed => {
            // User has claimed on-chain, update database
            let affected = premarket_repo::update_holder_claimed_status(
                pool,
                &premarket_pubkey.to_string(),
                user_wallet,
                true,
            )
            .await
            .map_err(ErrorInternalServerError)?;

            println!(
                "✅ Updated claimed status for user {} in premarket {}: affected {} rows",
                user_wallet, premarket_pubkey, affected
            );

            Ok((true, affected > 0))
        }
        Some(_user) => {
            // User exists but hasn't claimed yet
            println!(
                "ℹ️  User {} has not claimed tokens yet in premarket {}",
                user_wallet, premarket_pubkey
            );
            Ok((false, false))
        }
        None => {
            // User not found in premarket
            Err(actix_web::error::ErrorNotFound(format!(
                "User {} not found in premarket {}",
                user_wallet, premarket_pubkey
            )))
        }
    }
}
