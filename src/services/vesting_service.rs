use crate::models::vesting::{VestingHolderInfo, VestingInfo};
use crate::storage::vesting_repo;
use solana_sdk::signature::Signature;
use solana_sdk::pubkey::Pubkey;
use crate::models::premarket::SolanaNetwork;
use crate::services::solana_service_v2::vesting::generate_vesting_pda;
use actix_web::error::{ErrorInternalServerError, ErrorNotFound};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub enum VestingLookupType {
    PremarketId,
    PremarketAddress,
    VestingAddress,
    MintAddress,
}

pub async fn get_full_vesting_info(
    pool: &PgPool,
    key: &str,
    lookup_type: VestingLookupType,
) -> Result<Option<VestingInfo>, actix_web::Error> {
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
        VestingLookupType::MintAddress => vesting_repo::get_vesting_info_by_mint_address(pool, key).await,
    }
    .map_err(|e| ErrorInternalServerError(format!("Database error: {}", e)))?;

    let vesting_db = match vesting_db {
        Some(v) => v,
        None => return Ok(None),
    };

    let is_active = vesting_db.timestamp_start.is_some() && vesting_db.timestamp_end.is_some();

    Ok(Some(VestingInfo {
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
    }))
}

pub fn calculate_available_tokens(
    total: i64,
    claimed: i64,
    start: Option<i64>,
    end: Option<i64>,
    init_unlock_percent: i64,
    now: i64,
) -> Option<i64> {
    if start.is_none() || end.is_none() {
        return Some(0);
    }

    let start_ts = start.unwrap();
    let end_ts = end.unwrap();

    if now < start_ts {
        return Some(0);
    }
    if now >= end_ts {
        return Some(total - claimed);
    }

    let init_unlock_amount = (total * init_unlock_percent) / 100;

    let vesting_duration = end_ts - start_ts;
    let time_elapsed = now - start_ts;

    let vesting_amount = total - init_unlock_amount;

    let vested_amount = if vesting_duration > 0 {
        (vesting_amount * time_elapsed) / vesting_duration
    } else {
        vesting_amount
    };

    let total_available = init_unlock_amount + vested_amount;
    let available = total_available - claimed;

    Some(available.max(0))
}

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
            let vesting_db = vesting_repo::get_vesting_info_by_premarket_id(pool, premarket_id)
                .await
                .map_err(|e| ErrorInternalServerError(format!("Database error: {}", e)))?
                .ok_or_else(|| ErrorNotFound("Vesting not found"))?;

            let now = Utc::now().timestamp();
            let total_opt = if h.amount_token == 0 { None } else { Some(h.amount_token) };
            let claimed_opt = if h.claimed_amount_token == 0 { None } else { Some(h.claimed_amount_token) };
            let available_tokens = if let (Some(total), Some(claimed)) = (total_opt, claimed_opt) {
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
                amount_tokens: total_opt,
                claimed_tokens: claimed_opt,
                available_tokens,
                username: h.username,
                avatar_url: h.avatar_url,
            }))
        }
        None => Ok(None),
    }
}

pub async fn finalize_withdraw_vesting(
    pool: &PgPool,
    network: SolanaNetwork,
    signature: &Signature,
    token_mint: &str,
    holder_wallet: &str,
) -> Result<(), actix_web::Error> {
    println!(
        "[finalize_withdraw_vesting] start network={:?} sig={} mint={} wallet={}",
        network, signature, token_mint, holder_wallet
    );
    let vesting = get_full_vesting_info(pool, token_mint, VestingLookupType::MintAddress)
        .await?
        .ok_or_else(|| ErrorNotFound("Vesting not found for mint"))?;
    
    println!(
        "[vesting] premarket_id={} premarket_address={} vesting_address={} timestamp_start={:?} timestamp_end={:?} init_unlock={} is_active={}",
        vesting.premarket_id, vesting.premarket_address, vesting.vesting_address, vesting.timestamp_start, vesting.timestamp_end, vesting.init_unlock, vesting.is_active
    );

    let holder = vesting_repo::get_holder_by_wallet(pool, vesting.premarket_id, holder_wallet)
        .await
        .map_err(|e| ErrorInternalServerError(format!("Database error: {}", e)))?
        .ok_or_else(|| ErrorNotFound("Holder not found in premarket"))?;

    println!(
        "[holder] holder_wallet={} amount_lamport={} amount_token={} claimed_amount_token={}",
        holder.holder_wallet, holder.amount_lamport, holder.amount_token, holder.claimed_amount_token
    );

    let total_amount = holder.amount_token;
    println!(
        "[finalize_withdraw_vesting] before calc: premarket_id={} wallet={} total_amount={} claimed_prev={}",
        vesting.premarket_id,
        holder_wallet,
        total_amount,
        holder.claimed_amount_token
    );

    let claimed_now = calculate_claimed_tokens_delta_from_tx(
        network,
        signature,
        token_mint,
        holder_wallet,
    )
    .await?;
    println!(
        "[finalize_withdraw_vesting] claimed_now_from_tx={}",
        claimed_now
    );
    let claimed_next = holder.claimed_amount_token.saturating_add(claimed_now);
    println!(
        "[finalize_withdraw_vesting] claimed_next={}",
        claimed_next
    );

    vesting_repo::update_holder_token_amounts(
        pool,
        vesting.premarket_id,
        holder_wallet,
        total_amount,
        claimed_next,
    )
    .await
    .map_err(|e| ErrorInternalServerError(format!("Database error: {}", e)))?;

    Ok(())
}

pub async fn calculate_claimed_tokens_delta_from_tx(
    network: SolanaNetwork,
    signature: &Signature,
    token_mint: &str,
    holder_wallet: &str,
) -> Result<i64, actix_web::Error> {
    println!(
        "[calculate_claimed_tokens_delta_from_tx] start network={:?} sig={} mint={} wallet={}",
        network, signature, token_mint, holder_wallet
    );

    let delta_raw = crate::services::solana_service_v2::get_spl_token_delta(
        network,
        signature,
        token_mint,
        holder_wallet,
    )
    .await
    .map_err(|e| ErrorInternalServerError(format!("RPC tx delta error: {}", e)))?;

    println!(
        "[calculate_claimed_tokens_delta_from_tx] delta_raw={}",
        delta_raw
    );

    let claimed_now = if delta_raw > 0 {
        (delta_raw as i64).max(0)
    } else {
        0
    };

    println!(
        "[calculate_claimed_tokens_delta_from_tx] claimed_now={}",
        claimed_now
    );

    Ok(claimed_now)
}

/// Update vesting info for a premarket.
///
/// Semantics:
/// - If enabled=false: clear timestamp_start/end (vesting disabled) and update params.
/// - If enabled=true:
///   - ensure vesting_info exists (create if missing) **requires** `vesting_address` to be already set in DB.
///   - if timestamps are not set yet: set start=now, end=now+vesting_period_sec
///   - always update vesting_period/init_unlock
pub async fn update_vesting_info(
    pool: &PgPool,
    network: SolanaNetwork,
    premarket_id: Uuid,
    mint: Pubkey,
    enabled: bool,
    vesting_period_sec: i64,
    unlock_at_launch_percent: i64,
) -> Result<(), actix_web::Error> {
    if unlock_at_launch_percent < 0 || unlock_at_launch_percent > 100 {
        return Err(actix_web::error::ErrorBadRequest(
            "unlock_at_launch_percent must be 0..=100",
        ));
    }
    if enabled && vesting_period_sec <= 0 {
        return Err(actix_web::error::ErrorBadRequest(
            "vesting_period_sec must be > 0 when enabled=true",
        ));
    }

    // Ensure row exists (create minimal row if missing).
    // IMPORTANT: create_vesting_info requires vesting_address; we must have it somewhere.
    // If you don’t have it yet, either:
    //  - change create_vesting_info signature to accept Option<&str> and store NULL, or
    //  - add a separate repo method that does INSERT ... ON CONFLICT (premarket_id) DO NOTHING with provided address.
    let existing = vesting_repo::get_vesting_info_by_premarket_id(pool, premarket_id)
        .await
        .map_err(|e| ErrorInternalServerError(format!("Database error: {}", e)))?;

    if existing.is_none() {
        // try to read vesting_address from somewhere (for example, precomputed onchain address stored in DB)
        // You need a repo helper for that. If you already have it — replace this call.
        let vesting_address = generate_vesting_pda(network, mint);

        // create row with timestamps depending on enabled
        let now = Utc::now().timestamp();
        let (ts_start, ts_end) = if enabled {
            (Some(now), Some(now + vesting_period_sec))
        } else {
            (None, None)
        };

        vesting_repo::create_vesting_info(
            pool,
            premarket_id,
            &vesting_address.to_string(),
            vesting_period_sec,
            unlock_at_launch_percent,
            ts_start,
            ts_end,
        )
        .await
        .map_err(|e| ErrorInternalServerError(format!("Failed to create vesting info: {}", e)))?;

        return Ok(());
    }

    // Update existing row
    let now = Utc::now().timestamp();

    let current = existing.unwrap();

    let (timestamp_start, timestamp_end) = if !enabled {
        (None, None)
    } else {
        // if already started, keep existing start/end; else set new
        match (current.timestamp_start, current.timestamp_end) {
            (Some(s), Some(e)) => (Some(s), Some(e)),
            _ => (Some(now), Some(now + vesting_period_sec)),
        }
    };

    vesting_repo::update_vesting_info(
        pool,
        premarket_id,
        vesting_period_sec,
        unlock_at_launch_percent,
        timestamp_start,
        timestamp_end,
    )
    .await
    .map_err(|e| ErrorInternalServerError(format!("Failed to update vesting info: {}", e)))?;

    Ok(())
}
