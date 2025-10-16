
use crate::models::premarket::{PremarketState, PremarketListResult, TokenDynamicInfo, HolderInfo, LinkType, CommunityLink, CommunityInfoServiceModel, FullPremarketInfo, PremarketInfoServiceModel, TokenInfo, TokenLinks, PremarketGoal, UserInfoShort, JoinConfirmationStatusDTO, OutConfirmationStatusDTO, TxConfirmationStatusDTO, PremarketOnchainUser, PremarketOnchainData};

use crate::storage::models::{HolderDbModel, CommunityInfoDbModel, CommunityLinkDbModel, PremarketInfoDbModel};
use crate::storage::premarket_repo;
use sqlx::PgPool;
use actix_web::error::ErrorInternalServerError;
use uuid::Uuid;
use chrono::Utc;
use actix_web::error::ErrorBadRequest;

use crate::constants::CRYPTO_PRICE_API_URL;

use solana_client::nonblocking::rpc_client::RpcClient; // CHANGED
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

pub async fn get_full_premarket_info(
    pool: &PgPool,
    bc_address: &str,
) -> Result<Option<FullPremarketInfo>, actix_web::Error> {

    match premarket_repo::get_premarket_info_by_bc_address(pool, bc_address).await {
        Ok(Some((pm_db, cm_db, links_db))) => {
            let premarket_info = PremarketInfoServiceModel {
                id: Some(pm_db.id),
                blockchain_address: pm_db.bc_address,
                creator: UserInfoShort{
                    id: Some(pm_db.creator_id),
                    blockchain_address: pm_db.creator_address
                },
                token_info: TokenInfo{
                    address: pm_db.mint_address,
                    name: pm_db.name,
                    description: pm_db.description,
                    symbol: pm_db.symbol,
                    image_url: pm_db.image_url,
                    data_uri: pm_db.data_uri,
                    links: TokenLinks{
                        telegram: pm_db.telegram,
                        twitter: pm_db.twitter, 
                        web_site: pm_db.web_site
                    }
                },
                state: pm_db.state.into(),
                goal: PremarketGoal{
                    percent: pm_db.premarket_goal_pers,
                    solana_lamp: pm_db.premarket_goal_sol_lamp,
                },
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
                links: links
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
) -> Result<Option<PremarketListResult>, actix_web::Error> {
    let res = premarket_repo::get_list(pool, cursor, limit)
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
        .map(|pm_db| {
            PremarketInfoServiceModel {
                id: Some(pm_db.id),
                blockchain_address: pm_db.bc_address,
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
                    percent: pm_db.premarket_goal_pers,
                    solana_lamp: pm_db.premarket_goal_sol_lamp,
                },
                deadline_timestamp: pm_db.premarket_deadline,
                created_timestamp: pm_db.premarket_created,
                finished_timestamp: pm_db.premarket_finished,
            }
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
        premarket_goal_pers: premarket.goal.percent,
        premarket_goal_sol_lamp: premarket.goal.solana_lamp,
        premarket_deadline: premarket.deadline_timestamp,
        premarket_created: premarket.created_timestamp,
        premarket_finished: premarket.finished_timestamp,
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
        bc_address,                                // было &bc_address
        &community.description,
        community.token_banner_url.as_deref(),     // Option<String> -> Option<&str>
        link_db,                                   // было &link_db
    )
    .await
    .map_err(ErrorInternalServerError)
}

pub async fn get_dynamic_info(
    pool: &PgPool,
    premarket_pubkey: &str,
) -> Result<TokenDynamicInfo, actix_web::Error> {
    let holder_limit = 300;
    let holder_data = premarket_repo::get_holders_by_premarket_address(pool, premarket_pubkey, holder_limit)
        .await
        .map_err(ErrorInternalServerError)?;

    let holder_service_list: Vec<HolderInfo> = holder_data.holders
        .into_iter()
        .map(|h| HolderInfo {
            id: h.holder_id,
            wallet_address: h.holder_wallet,
            join_timestamp: h.join_timestamp,
            icon_url: h.avatar_url,
            username: h.username,
            amount_sol_lamp: h.amount_lamport as u64,
        })
        .collect();

    let current_price_lamp = get_price_by_market_cap(holder_data.reserved_sol_lamp as u64).await;
    println!("reserved_sol_lamp: {}", holder_data.reserved_sol_lamp);
    println!("current_price_lamp: {}", current_price_lamp);
    let price_24h_ago_lamp = get_price_by_market_cap(holder_data.reserved_sol_24h_before_lamp as u64).await;

    let change_24h = if price_24h_ago_lamp > 0.0 {
        ((current_price_lamp - price_24h_ago_lamp) / price_24h_ago_lamp) * 100.0
    } else {
        100.0
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
) -> Result<(), actix_web::Error>  {
    let affected = premarket_repo::update_premarket_state(
        pool,
        premarket_pubkey,
        &new_state.to_string(),
        premarket_finished
    )
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    if affected == 0 {
        return Err(actix_web::error::ErrorNotFound(
            format!("premarket '{}' not found", premarket_pubkey),
        ));
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
        holder_id:holder.id,
        join_timestamp:holder.join_timestamp,
        amount_lamport: holder.amount_sol_lamp as i64,
        out_timestamp: None,
        avatar_url: None, 
        username: None
    };
    premarket_repo::insert_holder(
        pool,
        premarket_pubkey,
        &holder
    )
    .await
    .map_err(actix_web::error::ErrorInternalServerError)
}

pub async fn remove_holder(
    pool: &PgPool,
    premarket_pubkey: &str,
    wallet_address: &str,
) -> Result<(), actix_web::Error> {
    premarket_repo::soft_delete_holder(pool, premarket_pubkey, wallet_address, Utc::now().timestamp())
        .await
        .map(|_| ()) // игнорируем u64, возвращаем ()
        .map_err(actix_web::error::ErrorInternalServerError)
}

pub async fn get_price_by_market_cap(reserved_sol_lamp: u64) -> f64 {
    let url = format!("{}ids=solana&vs_currencies=usd", CRYPTO_PRICE_API_URL);
    println!("Fetching SOL price from: {}", url);

    let current_sol_price = match reqwest::get(&url).await {
        Ok(response) => {
            println!("API response status: {}", response.status());
            match response.json::<serde_json::Value>().await {
                Ok(json) => {
                    println!("API response JSON: {}", json);
                    let price = json["solana"]["usd"].as_f64().unwrap_or(0.0);
                    println!("Extracted SOL price: {}", price);
                    price
                },
                Err(e) => {
                    println!("JSON parsing error: {:?}", e);
                    0.0
                }
            }
        },
        Err(e) => {
            println!("HTTP request error: {:?}", e);
            0.0
        }
    };
    
    println!("Final current_sol_price: {}", current_sol_price);
    let price = reserved_sol_lamp as f64 / 1_000_000_000_000_000.0 * current_sol_price;
    let final_price = (price * 1_000_000.0).round() / 1_000_000.0;
    println!("Calculated price: {} (reserved_sol_lamp: {}, final_price: {})", final_price, reserved_sol_lamp, final_price);
    final_price
}

pub async fn get_tx_confirmation_status(
    client: &RpcClient,
    tx_id: &str
) -> Result<TxConfirmationStatusDTO, actix_web::Error> {
    let tx_signature = match solana_sdk::signature::Signature::from_str(tx_id) {
        Ok(sig) => sig,
        Err(_) => return Err(actix_web::error::ErrorBadRequest("Invalid transaction signature format")),
    };

    let status = client
        .get_signature_status(&tx_signature)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to get transaction status: {}", e)))?;


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
) -> Result<JoinConfirmationStatusDTO, actix_web::Error>  {
    let user_pubkey = Pubkey::from_str(user).map_err(|_| actix_web::error::ErrorBadRequest("Invalid user public key"))?;
    let premarket_pubkey = Pubkey::from_str(premarket_account).map_err(|_| actix_web::error::ErrorBadRequest("Invalid premarket account public key"))?;

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
) -> Result<OutConfirmationStatusDTO, actix_web::Error>  {
    let user_pubkey = Pubkey::from_str(user).map_err(|_| actix_web::error::ErrorBadRequest("Invalid user public key"))?;
    let premarket_pubkey = Pubkey::from_str(premarket_account).map_err(|_| actix_web::error::ErrorBadRequest("Invalid premarket account public key"))?;

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
    premarket_account: &Pubkey
) -> Result<PremarketOnchainData, actix_web::Error> {
    let account = client
        .get_account(premarket_account)
        .await
        .map_err(|e| ErrorInternalServerError(format!("Failed to fetch account: {e}")))?;

    // Minimum length: 8 discriminator + 4 length + (could be zero users) + 8+8+8+32 for tail fields
    if account.data.len() < 8 + 4 + 8 + 8 + 8 + 32 {
        return Err(ErrorBadRequest("Account data too short for premarket layout"));
    }

    // Skip discriminator
    let data = &account.data[8..];

    // Read users vector length
    let users_len_bytes: [u8; 4] = data[0..4].try_into().unwrap();
    let users_len = u32::from_le_bytes(users_len_bytes) as usize;

    // Each user entry: 32 (pubkey) + 8 (lamports)
    let users_section_len = users_len.checked_mul(40)
        .ok_or_else(|| ErrorBadRequest("Users length overflow"))?;

    let needed_len = 4 + users_section_len + (8 + 8 + 8 + 32); // vec length + users + tail fields
    if data.len() < needed_len {
        return Err(ErrorBadRequest("Account data too short for declared users length"));
    }

    let mut users = Vec::with_capacity(users_len);
    let mut offset = 4;
    for _ in 0..users_len {
        let pk_slice = &data[offset..offset+32];
        let wallet = Pubkey::new_from_array(pk_slice.try_into().unwrap());
        let lamports = u64::from_le_bytes(data[offset+32..offset+40].try_into().unwrap());
        users.push(PremarketOnchainUser {
            wallet,
            contributed_lamports: lamports,
        });
        offset += 40;
    }

    // Tail fields
    let end_timestamp = i64::from_le_bytes(data[offset..offset+8].try_into().unwrap());
    offset += 8;

    let goal_lamports = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap());
    offset += 8;

    let max_lamports = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap());
    offset += 8;

    let mint = Pubkey::new_from_array(data[offset..offset+32].try_into().unwrap());

    Ok(PremarketOnchainData {
        users,
        end_timestamp,
        goal_lamports,
        max_lamports,
        mint,
    })
}

