use crate::models::premarket::{
    BondingPostion, CommunityInfoServiceModel, CommunityLink, CreatePremarketConceptModel,
    CreatePremarketInfoServiceModel, FullPremarketInfo, HolderEntryData, HolderEntryInfo,
    HolderInfo, HolderWhitelistInfo, LinkType, PremarketGoal, PremarketInfoServiceModel,
    PremarketListResult, PremarketLookupKeyType, PremarketOnchainData, PremarketOnchainUser,
    PremarketState, SolanaNetwork, TokenDynamicInfo, TokenEntryInfo, TokenInfo, TokenLinks,
    TxConfirmationStatusDTO, UserInfoShort,
};
use crate::models::user::UserContextData;
use crate::services::solana_service_v2::generate_premarket_pda;
use crate::services::solana_service_v2::utils::parse_privkey_64;
use crate::services::user_service;
use crate::storage::signing_keys::{
    acquire_signing_key, get_mint_signing_keypair_by_premarket, update_premarket_pubkey,
};

use crate::storage::premarket_repo;
use crate::storage::vesting_repo;
use crate::storage::whitelist_repo;
use anyhow::Error as AnyhowError;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

use crate::models::vesting::VestingSettingsServiceModel;
use crate::services::solana_price_service;

pub const VIRTUAL_SUPPLY_RATIO: u64 = 30_000_000_000;
pub const VIRTUAL_TOKEN_RATIO: u64 = 1_073_000_000_000_000;

#[derive(Debug)]
pub enum PremarketServiceError {
    InvalidUuid,
    MissingUser,
    InvalidTransactionSignature,
    InvalidOnchainData(&'static str),
    NotFound(String),
    Storage(AnyhowError),
    Internal(String),
    Chain(String),
}

impl fmt::Display for PremarketServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUuid => write!(f, "invalid uuid format"),
            Self::MissingUser => write!(f, "user is required"),
            Self::InvalidTransactionSignature => write!(f, "invalid transaction signature format"),
            Self::InvalidOnchainData(message) => write!(f, "invalid onchain data: {message}"),
            Self::NotFound(message) => write!(f, "{message}"),
            Self::Storage(err) => write!(f, "storage error: {err}"),
            Self::Internal(message) => write!(f, "internal error: {message}"),
            Self::Chain(message) => write!(f, "chain error: {message}"),
        }
    }
}

impl Error for PremarketServiceError {}

impl From<AnyhowError> for PremarketServiceError {
    fn from(value: AnyhowError) -> Self {
        Self::Storage(value)
    }
}

pub type PremarketServiceResult<T> = Result<T, PremarketServiceError>;

pub async fn get_mint_keypair(pool: &PgPool, premarket: Pubkey) -> PremarketServiceResult<Keypair> {
    let pair = get_mint_signing_keypair_by_premarket(pool, &premarket.to_string())
        .await
        .map_err(|err| PremarketServiceError::Storage(AnyhowError::new(err)))?
        .ok_or_else(|| PremarketServiceError::NotFound(format!("mint key for {premarket}")))?;

    let mint_bytes = parse_privkey_64(&pair.priv_key).map_err(|err| {
        PremarketServiceError::Internal(format!("mint priv_key parse failed: {err}"))
    })?;

    Keypair::from_bytes(&mint_bytes).map_err(|err| {
        PremarketServiceError::Internal(format!("invalid mint keypair bytes: {err}"))
    })
}

pub async fn get_full_premarket_info(
    pool: &PgPool,
    key: &str,
    key_type: PremarketLookupKeyType,
) -> PremarketServiceResult<Option<FullPremarketInfo>> {
    let data = match key_type {
        PremarketLookupKeyType::BcAddress => {
            premarket_repo::get_premarket_info_by_bc_address(pool, key).await
        }
        PremarketLookupKeyType::Name => premarket_repo::get_premarket_info_by_name(pool, key).await,
        PremarketLookupKeyType::Id => {
            let pm_id = Uuid::parse_str(key).map_err(|_| PremarketServiceError::InvalidUuid)?;
            premarket_repo::get_premarket_info_by_id(pool, &pm_id).await
        }
    };

    match data {
        Ok(Some(premarket_with_community)) => {
            let vesting_settings = vesting_repo::get_vesting_info_by_premarket_id(
                pool,
                premarket_with_community.main_info.id,
            )
            .await
            .map_err(PremarketServiceError::Storage)?
            .map(|v| VestingSettingsServiceModel {
                enabled: v.timestamp_start.is_some() && v.timestamp_end.is_some(),
                vesting_period_sec: Some(v.vesting_period),
                unlock_at_launch_percent: Some(v.init_unlock),
            });

            Ok(Some(FullPremarketInfo {
                main_info: premarket_with_community.main_info,
                community: premarket_with_community.community,
                vesting_settings,
            }))
        }
        Ok(None) => Ok(None),
        Err(e) => Err(PremarketServiceError::Storage(e)),
    }
}

pub async fn get_user_concept(
    pool: &PgPool,
    creator_id: &Uuid,
) -> PremarketServiceResult<Option<PremarketInfoServiceModel>> {
    let data = premarket_repo::get_user_concept(pool, creator_id)
        .await
        .map_err(PremarketServiceError::Storage)?;

    Ok(data)
}

pub async fn get_main_premarket_info(
    pool: &PgPool,
    bc_address: &str,
) -> PremarketServiceResult<Option<PremarketInfoServiceModel>> {
    let data: Option<PremarketInfoServiceModel> =
        premarket_repo::get_main_premarket_info_by_bc_address(pool, bc_address)
            .await
            .map_err(PremarketServiceError::Storage)?;

    Ok(data)
}

pub async fn get_list(
    pool: &PgPool,
    cursor: i64,
    limit: i64,
    states: Option<Vec<PremarketState>>,
    only_user_token: bool,
    user_opt: Option<UserContextData>,
) -> PremarketServiceResult<Option<PremarketListResult>> {
    let user_id_opt = user_opt.as_ref().map(|u| u.internal_id);
    let state_names = states.map(|items| {
        items
            .into_iter()
            .map(|state| state.to_string())
            .collect::<Vec<_>>()
    });
    let state_names = state_names.filter(|items| !items.is_empty());

    let res = if only_user_token {
        let user_id = user_id_opt.ok_or(PremarketServiceError::MissingUser)?;
        premarket_repo::get_list_user_related(pool, user_id, cursor, limit, state_names)
            .await
            .map_err(PremarketServiceError::Storage)?
    } else {
        premarket_repo::get_list_public(pool, user_id_opt, cursor, limit, state_names)
            .await
            .map_err(PremarketServiceError::Storage)?
    };

    let (items, total) = res;

    Ok(Some(PremarketListResult {
        items,
        total: Some(total),
    }))
}

pub async fn create_concept(
    pool: &PgPool,
    network: SolanaNetwork,
    concept_data: &CreatePremarketConceptModel,
) -> PremarketServiceResult<(uuid::Uuid, String)> {
    let concept = get_user_concept(pool, &concept_data.creator.id).await?;
    let (concept_id, blockchain_address, mint_pubkey) = match concept {
        Some(c) => {
            let affected = premarket_repo::update_concept(pool, c.id, concept_data)
                .await
                .map_err(PremarketServiceError::Storage)?;

            if affected == 0 {
                return Err(PremarketServiceError::Internal(
                    "failed to update existing concept".to_string(),
                ));
            }

            return Ok((c.id, c.blockchain_address));
        }
        None => {
            let pm_uuid = uuid::Uuid::new_v4();
            let exp_keypair = acquire_signing_key(pool, &pm_uuid.to_string())
                .await
                .map_err(|e| {
                    PremarketServiceError::Internal(format!("acquire_signing_key failed: {e}"))
                })?;

            let keypair = match exp_keypair {
                Some(k) => k,
                None => {
                    return Err(PremarketServiceError::Internal(
                        "no mint keypair available".to_string(),
                    ))
                }
            };

            let pda = generate_premarket_pda(network, &keypair.priv_key)
                .await
                .map_err(|e| {
                    PremarketServiceError::Internal(format!("generate_premarket_pda failed: {e}"))
                })?;

            update_premarket_pubkey(pool, &pda.to_string(), &keypair.id)
                .await
                .map_err(|e| {
                    PremarketServiceError::Internal(format!("update_premarket_pubkey failed: {e}"))
                })?;

            (pm_uuid, pda.to_string(), keypair.pub_key)
        }
    };

    let premarket = CreatePremarketInfoServiceModel {
        id: Some(concept_id),
        blockchain_address: blockchain_address.clone(),
        goal: concept_data.goal.clone(),
        deadline_timestamp: concept_data.deadline_timestamp,
        created_timestamp: Utc::now().timestamp(),
        concept_created_timestamp: concept_data.concept_created_timestamp,
        finished_timestamp: None,
        is_extended: false,
        is_hided: concept_data.is_hided,
        is_concept_visible: concept_data.is_concept_visible,
        is_whitelist_enabled: false,
        state: PremarketState::Concept,
        short_url_name: None,
        creator: concept_data.creator.clone(),
        token_info: TokenInfo {
            address: mint_pubkey.clone(),
            name: concept_data.token_info.name.clone(),
            description: concept_data.token_info.description.clone(),
            symbol: concept_data.token_info.symbol.clone(),
            image_url: concept_data.token_info.image_url.clone(),
            data_uri: concept_data.token_info.data_uri.clone(),
            links: concept_data.token_info.links.clone(),
        },
    };
    let community = CommunityInfoServiceModel {
        description: "".to_string(),
        token_banner_url: None,
        links: None,
    };
    let premarket_id = create_full_premarket_info(&pool, premarket, community).await?;

    Ok((premarket_id, blockchain_address))
}

pub async fn create_full_premarket_info(
    pool: &PgPool,
    premarket: CreatePremarketInfoServiceModel,
    community: CommunityInfoServiceModel,
) -> PremarketServiceResult<uuid::Uuid> {
    let pm_id = premarket.id.unwrap_or_else(uuid::Uuid::new_v4);

    premarket_repo::create_premarket_and_community(pool, &premarket, &community)
        .await
        .map_err(PremarketServiceError::Storage)?;

    Ok(pm_id)
}

pub async fn update_availability_info(
    pool: &PgPool,
    premarket_pubkey: &str,
    is_hided: Option<bool>,
    is_concept_visible: Option<bool>,
    is_whitelist_enabled: Option<bool>,
    short_url_name: Option<String>,
) -> PremarketServiceResult<()> {
    premarket_repo::update_availability_info(
        pool,
        premarket_pubkey,
        is_hided,
        is_concept_visible,
        is_whitelist_enabled,
        short_url_name,
    )
    .await
    .map_err(PremarketServiceError::Storage)
}

pub async fn update_community_info(
    pool: &PgPool,
    bc_address: &str,
    community: CommunityInfoServiceModel,
) -> PremarketServiceResult<()> {
    let links = community.links;

    premarket_repo::update_community_info(
        pool,
        bc_address, // было &bc_address
        &community.description,
        community.token_banner_url.as_deref(), // Option<String> -> Option<&str>
        links,
    )
    .await
    .map_err(PremarketServiceError::Storage)
}

pub async fn get_dynamic_info(
    pool: &PgPool,
    premarket_pubkey: &str,
) -> PremarketServiceResult<TokenDynamicInfo> {
    let holder_limit = 300;
    let holder_data =
        premarket_repo::get_holders_by_premarket_address(pool, premarket_pubkey, holder_limit)
            .await
            .map_err(PremarketServiceError::Storage)?;

    let vesting_db = vesting_repo::get_vesting_info_by_premarket_address(pool, premarket_pubkey)
        .await
        .map_err(PremarketServiceError::Storage)?;

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

    let now = Utc::now().timestamp();
    let total_tokens_i64 = holder_data.total_token_amount.max(0);
    let total_claimed_i64 = holder_data
        .total_claimed_token_amount
        .max(0)
        .min(total_tokens_i64);
    let total_tokens = total_tokens_i64 as u64;
    let total_claimed = total_claimed_i64 as u64;

    let vested_total = calculate_vested_dec(
        total_tokens_i64,
        total_claimed_i64,
        vesting_db
            .as_ref()
            .map(|v| (v.timestamp_start, v.timestamp_end, v.init_unlock)),
        now,
    );

    let vesting_info = if total_tokens == 0 && total_claimed == 0 {
        None
    } else {
        let (starttime_ms, endtime_ms) = match vesting_db.as_ref() {
            Some(v) => (
                v.timestamp_start.map(|v| v * 1000),
                v.timestamp_end.map(|v| v * 1000),
            ),
            None => (None, None),
        };
        Some(crate::models::premarket::DynamicVestingInfo {
            starttime_ms,
            endtime_ms,
            entry: TokenEntryInfo {
                total_dec: total_tokens,
                vested_dec: vested_total,
                claimed_dec: total_claimed,
            },
        })
    };

    Ok(TokenDynamicInfo {
        holders_count: holder_data.total_active_count as u32,
        current_price_lamp,
        reserved_sol_lamp: holder_data.reserved_sol_lamp as u64,
        change_24h,
        holders: holder_data.holders,
        vesting_info,
    })
}

pub async fn get_holder_entry_info(
    pool: &PgPool,
    premarket_pubkey: &str,
    holder_pubkey: &str,
) -> PremarketServiceResult<HolderEntryInfo> {
    let premarket_id = match premarket_repo::get_premarket_id_by_bc_address(pool, premarket_pubkey)
        .await
        .map_err(PremarketServiceError::Storage)?
    {
        Some(id) => id,
        None => {
            return Err(PremarketServiceError::NotFound(
                "premarket not found".to_string(),
            ))
        }
    };

    let pm_opt = premarket_repo::get_main_premarket_info_by_bc_address(pool, premarket_pubkey)
        .await
        .map_err(PremarketServiceError::Storage)?;
    let pm = match pm_opt {
        Some(pm) => pm,
        None => {
            return Err(PremarketServiceError::NotFound(
                "premarket not found".to_string(),
            ))
        }
    };

    let holder_entry_data_opt =
        premarket_repo::get_holder_entry_by_premarket_id(pool, premarket_id, holder_pubkey)
            .await
            .map_err(PremarketServiceError::Storage)?;

    let whitelist = {
        if pm.is_whitelist_enabled {
            let user_opt = user_service::get_by_wallet_address(pool, holder_pubkey)
                .await
                .map_err(|e| PremarketServiceError::Internal(format!("user lookup failed: {e}")))?;
            if let Some(user) = user_opt {
                whitelist_repo::get_status_with_updated_at(pool, premarket_id, user.id)
                    .await
                    .map_err(PremarketServiceError::Storage)?
                    .map(|(status, updated_at)| HolderWhitelistInfo {
                        status: status.as_str().to_string(),
                        updated_at,
                    })
            } else {
                None
            }
        } else {
            None
        }
    };

    let entry = if let Some(holder_entry_data) = holder_entry_data_opt {
        let calculated_amount_token = calculate_token_amount(holder_entry_data) as i64;
        let amount_token_i64 = if holder_entry_data.amount_token > 0 {
            holder_entry_data.amount_token
        } else {
            calculated_amount_token
        };
        let amount_token_i64 = amount_token_i64.max(0);

        let claimed_token_i64 = holder_entry_data
            .claimed_amount_token
            .max(0)
            .min(amount_token_i64);

        let vesting = vesting_repo::get_vesting_info_by_premarket_id(pool, premarket_id)
            .await
            .map_err(PremarketServiceError::Storage)?;

        let vested_dec = calculate_vested_dec(
            amount_token_i64,
            claimed_token_i64,
            vesting
                .as_ref()
                .map(|v| (v.timestamp_start, v.timestamp_end, v.init_unlock)),
            Utc::now().timestamp(),
        );

        Some(HolderEntryData {
            amount_sol_lamp: holder_entry_data.amount_sol_lamp,
            token: TokenEntryInfo {
                total_dec: amount_token_i64 as u64,
                vested_dec: vested_dec,
                claimed_dec: claimed_token_i64 as u64,
            },
            rank: holder_entry_data.rank,
        })
    } else {
        None
    };

    Ok(HolderEntryInfo { entry, whitelist })
}

fn calculate_vested_dec(
    total_tokens: i64,
    claimed_tokens: i64,
    vesting: Option<(Option<i64>, Option<i64>, i64)>,
    now: i64,
) -> u64 {
    let total = total_tokens.max(0);
    let claimed = claimed_tokens.max(0).min(total);

    let vested = match vesting {
        Some((start, end, init_unlock)) => {
            let available = crate::services::vesting_service::calculate_available_tokens(
                total,
                claimed,
                start,
                end,
                init_unlock,
                now,
            )
            .unwrap_or(0)
            .max(0);

            claimed.saturating_add(available).min(total)
        }
        None => total,
    };

    vested as u64
}

pub async fn set_premarket_state(
    pool: &PgPool,
    premarket_pubkey: &str,
    new_state: PremarketState,
    update_time: i64,
) -> PremarketServiceResult<()> {
    let affected = match new_state {
        PremarketState::Canceled | PremarketState::Finished => {
            premarket_repo::update_premarket_state_to_finish(
                pool,
                premarket_pubkey,
                &new_state.to_string(),
                update_time,
            )
            .await
            .map_err(PremarketServiceError::Storage)?
        }
        PremarketState::Premarket => premarket_repo::update_premarket_state_to_start(
            pool,
            premarket_pubkey,
            &new_state.to_string(),
            update_time,
        )
        .await
        .map_err(PremarketServiceError::Storage)?,
        PremarketState::Concept => {
            premarket_repo::update_premarket_state(pool, premarket_pubkey, &new_state.to_string())
                .await
                .map_err(PremarketServiceError::Storage)?
        }
    };

    if affected == 0 {
        return Err(PremarketServiceError::NotFound(format!(
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
) -> PremarketServiceResult<()> {
    premarket_repo::insert_holder(pool, premarket_pubkey, &holder)
        .await
        .map_err(PremarketServiceError::Storage)
}

pub async fn remove_holder(
    pool: &PgPool,
    premarket_pubkey: &str,
    wallet_address: &str,
) -> PremarketServiceResult<()> {
    premarket_repo::soft_delete_holder(
        pool,
        premarket_pubkey,
        wallet_address,
        Utc::now().timestamp(),
    )
    .await
    .map(|_| ()) // игнорируем u64, возвращаем ()
    .map_err(PremarketServiceError::Storage)
}

pub async fn get_price_by_market_cap(real_lamp_amount: u64) -> f64 {
    let current_sol_price = solana_price_service::get_sol_price().await;

    let real_sol_amount: f64 = real_lamp_amount as f64 / 1_000_000_000.0;

    let real_token_bought_amount: u64 =
        ((1_073_000_000.0 * real_sol_amount) / (30.0 + real_sol_amount)) as u64;
    let real_token_amount: u64 = 793_100_000 - real_token_bought_amount;

    let virtual_lamp_amount: f64 = real_sol_amount + 30.0;
    let virtual_token_amount = real_token_amount + 279_900_000;

    let price = virtual_lamp_amount / virtual_token_amount as f64 * current_sol_price;
    let final_price = (price * 1_000_000_000.0).round() / 1_000_000_000.0;

    // tracing::info!("Real sol: {}, token: {}, Virtual sol: {}, token: {} => price {}, Final price: {}", real_sol_amount, real_token_amount, virtual_lamp_amount, virtual_token_amount, price, final_price  );
    final_price
}

pub async fn get_tx_confirmation_status(
    client: &RpcClient,
    tx_id: &str,
) -> PremarketServiceResult<TxConfirmationStatusDTO> {
    let tx_signature = match solana_sdk::signature::Signature::from_str(tx_id) {
        Ok(sig) => sig,
        Err(_) => return Err(PremarketServiceError::InvalidTransactionSignature),
    };

    let status = client
        .get_signature_status(&tx_signature)
        .await
        .map_err(|e| {
            PremarketServiceError::Chain(format!("failed to get transaction status: {e}"))
        })?;

    match status {
        Some(status) if status.is_ok() => Ok(TxConfirmationStatusDTO::Confirmed),
        Some(_) => Ok(TxConfirmationStatusDTO::Failed),
        None => Ok(TxConfirmationStatusDTO::Pending),
    }
}

pub async fn get_premarket_data(
    client: &RpcClient,
    premarket_account: &Pubkey,
) -> PremarketServiceResult<PremarketOnchainData> {
    let account = client
        .get_account(premarket_account)
        .await
        .map_err(|e| PremarketServiceError::Chain(format!("failed to fetch account: {e}")))?;

    // Minimum length: 8 discriminator + 4 length + (could be zero users) + 8+1+8+8+32 for tail fields
    if account.data.len() < 8 + 4 + 8 + 1 + 8 + 8 + 32 {
        return Err(PremarketServiceError::InvalidOnchainData(
            "account data too short for premarket layout",
        ));
    }

    // Skip discriminator
    let data = &account.data[8..];

    // Read users vector length
    let users_len_bytes: [u8; 4] = data[0..4].try_into().unwrap();
    let users_len = u32::from_le_bytes(users_len_bytes) as usize;

    // Each user entry: 32 (pubkey) + 8 (lamports) + 1 (claimed)
    let users_section_len =
        users_len
            .checked_mul(41)
            .ok_or(PremarketServiceError::InvalidOnchainData(
                "users length overflow",
            ))?;

    let needed_len = 4 + users_section_len + (8 + 1 + 8 + 8 + 32); // vec length + users + tail fields (end_timestamp + extended_premarket + goal + max + mint)
    if data.len() < needed_len {
        return Err(PremarketServiceError::InvalidOnchainData(
            "account data too short for declared users length",
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
) -> PremarketServiceResult<Option<f64>> {
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
        Err(e) => return Err(PremarketServiceError::Storage(e)),
    };

    // Get the total lamports collected before this holder joined
    let lamports_before_join =
        premarket_repo::get_lamports_before_timestamp(pool, premarket_pubkey, join_timestamp)
            .await
            .map_err(PremarketServiceError::Storage)?;
    let final_price = calculate_entry_price(lamports_before_join as u64).await;

    Ok(Some(final_price))
}

async fn calculate_entry_price(real_lamp_amount: u64) -> f64 {
    let current_sol_price = solana_price_service::get_sol_price().await;

    let real_sol_amount: f64 = real_lamp_amount as f64 / 1_000_000_000.0;

    let real_token_bought_amount: u64 =
        ((1_073_000_000.0 * real_sol_amount) / (30.0 + real_sol_amount)) as u64;
    let real_token_amount: u64 = 793_100_000 - real_token_bought_amount;

    let virtual_lamp_amount: f64 = real_sol_amount + 30.0;
    let virtual_token_amount = real_token_amount + 279_900_000;

    let price = virtual_lamp_amount / virtual_token_amount as f64 * current_sol_price;
    let final_price = (price * 1_000_000_000.0).round() / 1_000_000_000.0;
    final_price
}

pub async fn update_premarket_deadline(
    pool: &PgPool,
    premarket_pubkey: &str,
    new_deadline: i64,
) -> PremarketServiceResult<()> {
    let affected = premarket_repo::update_premarket_deadline(pool, premarket_pubkey, new_deadline)
        .await
        .map_err(PremarketServiceError::Storage)?;

    if affected == 0 {
        return Err(PremarketServiceError::NotFound(format!(
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
) -> PremarketServiceResult<()> {
    let affected = premarket_repo::update_all_links_premarket(
        pool,
        premarket_pubkey,
        image_url,
        data_uri,
        telegram,
        twitter,
        web_site,
    )
    .await
    .map_err(PremarketServiceError::Storage)?;

    if affected == 0 {
        return Err(PremarketServiceError::NotFound(format!(
            "premarket '{}' not found",
            premarket_pubkey
        )));
    }

    Ok(())
}

pub async fn update_premarket_uri(
    pool: &PgPool,
    premarket_id: &Uuid,
    new_uri: &String,
    new_image_url: &String,
) -> PremarketServiceResult<()> {
    let affected = premarket_repo::update_premarket_uri(pool, premarket_id, new_uri, new_image_url)
        .await
        .map_err(PremarketServiceError::Storage)?;

    if affected == 0 {
        return Err(PremarketServiceError::NotFound(format!(
            "premarket '{}' not found",
            premarket_id
        )));
    }

    Ok(())
}

pub async fn user_claimed_token(
    pool: &PgPool,
    premarket_pubkey: &Pubkey,
    user_wallet: &str,
) -> PremarketServiceResult<()> {
    let affected = premarket_repo::update_holder_claimed_status(
        pool,
        &premarket_pubkey.to_string(),
        user_wallet,
        true,
    )
    .await
    .map_err(PremarketServiceError::Storage)?;

    if affected == 0 {
        return Err(PremarketServiceError::NotFound(format!(
            "premarket '{}' or holder '{}' not found",
            premarket_pubkey, user_wallet
        )));
    }

    Ok(())
}

fn calculate_token_amount(input: BondingPostion) -> u64 {
    let amount_sol_in_curve_lamp = get_in_curve(input.amount_sol_lamp);
    let before_amount_sol_in_curve_lamp = get_in_curve(input.before_amount_sol_lamp);

    let mut vsr = VIRTUAL_SUPPLY_RATIO;
    let mut vtr = VIRTUAL_TOKEN_RATIO;

    let user_tokens_before = tokens_out_from_sol(before_amount_sol_in_curve_lamp, vsr, vtr);

    vsr = vsr.saturating_add(before_amount_sol_in_curve_lamp);
    vtr = vtr.saturating_sub(user_tokens_before);

    tokens_out_from_sol(amount_sol_in_curve_lamp, vsr, vtr)
}

fn get_in_curve(amount: u64) -> u64 {
    return get_after_pump_fee(get_after_revelcy_fee(amount));
}

fn get_after_pump_fee(amount: u64) -> u64 {
    return amount.checked_mul(985).unwrap().checked_div(1000).unwrap();
}

fn get_after_revelcy_fee(amount: u64) -> u64 {
    return amount.checked_mul(985).unwrap().checked_div(1000).unwrap();
}

/// Given:
/// - `sol_in`               = incoming SOL in lamports (u64)
/// - `virtual_sol_reserves` = SOL-side reserve in lamports (u64)
/// - `virtual_token_reserves` = token-side reserve in smallest units (u64)
/// Returns how many tokens (in smallest units) you’ll receive.
/// ΔY = Y * ΔX / (X + ΔX)
///   where:
///     X  = virtual_sol_reserves (SOL‐side reserve, in lamports)
///     Y  = virtual_token_reserves (token‐side reserve, in smallest units)
///     ΔX = sol_in (incoming SOL, in lamports)
fn tokens_out_from_sol(sol_in: u64, virtual_sol_reserves: u64, virtual_token_reserves: u64) -> u64 {
    // Do multiplication in u128 to avoid overflow
    let numerator: u128 = (virtual_token_reserves as u128) * (sol_in as u128);
    let denominator: u128 = (virtual_sol_reserves as u128) + (sol_in as u128);
    // Floor division gives the integer token amount
    (numerator / denominator) as u64
}
