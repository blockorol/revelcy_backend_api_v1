use crate::models::premarket::{VestingHolderInfo, VestingInfo};
use crate::storage::vesting_repo;
use actix_web::error::{ErrorInternalServerError, ErrorNotFound};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

/// Lookup type for vesting info queries
#[derive(Debug, Clone, Copy)]
pub enum VestingLookupType {
    PremarketId,
    PremarketAddress,
    VestingAddress,
    MintAddress,
}

/// Get vesting info (without holders)
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `key` - The lookup key (UUID string for PremarketId, address string for others)
/// * `lookup_type` - Type of lookup to perform
///
/// # Returns
/// Vesting information, or None if not found
pub async fn get_full_vesting_info(
    pool: &PgPool,
    key: &str,
    lookup_type: VestingLookupType,
) -> Result<Option<VestingInfo>, actix_web::Error> {
    // Get vesting info based on lookup type
    let vesting_db = match lookup_type {
        VestingLookupType::PremarketId => {
            let uuid = Uuid::parse_str(key)
                .map_err(|e| ErrorInternalServerError(format!("Invalid UUID: {}", e)))?;
            vesting_repo::get_vesting_info_by_premarket_id(pool, uuid).await
        }
        VestingLookupType::PremarketAddress => {
            vesting_repo::get_vesting_info_by_premarket_address(pool, key).await
        }
        VestingLookupType::VestingAddress => {
            vesting_repo::get_vesting_info_by_vesting_address(pool, key).await
        }
        VestingLookupType::MintAddress => {
            vesting_repo::get_vesting_info_by_mint_address(pool, key).await
        }
    }
    .map_err(|e| ErrorInternalServerError(format!("Database error: {}", e)))?;

    // If no vesting found, return None
    let vesting_db = match vesting_db {
        Some(v) => v,
        None => return Ok(None),
    };

    // Calculate if vesting is active
    let is_active = vesting_db.timestamp_start.is_some() 
        && vesting_db.timestamp_end.is_some();

    // Convert to service model
    let vesting_info = VestingInfo {
        vesting_id: vesting_db.vesting_id,
        vesting_address: vesting_db.vesting_address,
        premarket_id: vesting_db.premarket_id,
        premarket_address: vesting_db.premarket_address,
        mint_address: vesting_db.mint_address,
        creator_id: vesting_db.creator_id,
        creator_address: vesting_db.creator_address,
        vesting_period: vesting_db.vesting_period,
        init_unlock: vesting_db.init_unlock,
        timestamp_start: vesting_db.timestamp_start,
        timestamp_end: vesting_db.timestamp_end,
        is_active,
    };

    Ok(Some(vesting_info))
}

/// Calculate available tokens for a holder based on vesting schedule
/// 
/// Formula:
/// - If vesting not started: 0
/// - At start: init_unlock% of total
/// - Linear vesting from start to end for remaining tokens
/// - After end: 100% of total
pub fn calculate_available_tokens(
    total: i64,
    claimed: i64,
    start: Option<i64>,
    end: Option<i64>,
    init_unlock_percent: i64,
    now: i64,
) -> Option<i64> {
    // If vesting hasn't started, no tokens available
    if start.is_none() || end.is_none() {
        return Some(0);
    }

    let start_ts = start.unwrap();
    let end_ts = end.unwrap();

    // If current time is before start, no tokens available
    if now < start_ts {
        return Some(0);
    }

    // If current time is after end, all tokens available
    if now >= end_ts {
        return Some(total - claimed);
    }

    // Calculate initial unlock amount
    let init_unlock_amount = (total * init_unlock_percent) / 100;

    // Linear vesting calculation
    let vesting_duration = end_ts - start_ts;
    let time_elapsed = now - start_ts;
    
    // Remaining tokens after initial unlock
    let vesting_amount = total - init_unlock_amount;
    
    // Calculate vested amount (linear)
    let vested_amount = if vesting_duration > 0 {
        (vesting_amount * time_elapsed) / vesting_duration
    } else {
        vesting_amount
    };

    // Total available = initial unlock + vested amount - already claimed
    let total_available = init_unlock_amount + vested_amount;
    let available = total_available - claimed;
    
    // Ensure we don't return negative
    Some(available.max(0))
}

/// Get vesting info for a specific holder (from premarket_holders)
pub async fn get_vesting_holder_info(
    pool: &PgPool,
    premarket_id: Uuid,
    holder_wallet: &str,
) -> Result<Option<VestingHolderInfo>, actix_web::Error> {
    let holder_db = vesting_repo::get_holder_by_wallet(pool, premarket_id, holder_wallet)
        .await
        .map_err(|e| ErrorInternalServerError(format!("Database error: {}", e)))?;

    match holder_db {
        Some(h) => {
            // Get vesting info to calculate available tokens
            let vesting_db = vesting_repo::get_vesting_info_by_premarket_id(pool, premarket_id)
                .await
                .map_err(|e| ErrorInternalServerError(format!("Database error: {}", e)))?
                .ok_or_else(|| ErrorNotFound("Vesting not found"))?;

            let now = Utc::now().timestamp();
            let available_tokens = if let (Some(total), Some(claimed)) = (h.amount_token, h.claimed_amount_token) {
                calculate_available_tokens(
                    total,
                    claimed,
                    vesting_db.timestamp_start,
                    vesting_db.timestamp_end,
                    vesting_db.init_unlock,
                    now,
                )
            } else {
                None
            };

            Ok(Some(VestingHolderInfo {
                holder_id: h.holder_id,
                holder_wallet: h.holder_wallet,
                amount_sol_lamp: h.amount_lamport,
                amount_tokens: h.amount_token,
                claimed_tokens: h.claimed_amount_token,
                available_tokens,
                username: h.username,
                avatar_url: h.avatar_url,
            }))
        }
        None => Ok(None),
    }
}
