use sqlx::PgPool;
use uuid::Uuid;
use anyhow::Result;

use super::models::{
    FullVestingInfoDbModel, HolderDbModel,
    VestingInfoDbModel,
};
use crate::models::premarket::VestingInfo;

/// Get vesting info by premarket_id
pub async fn get_vesting_info_by_premarket_id(
    pool: &PgPool,
    premarket_id: Uuid,
) -> Result<Option<FullVestingInfoDbModel>, sqlx::Error> {
    sqlx::query_as::<_, FullVestingInfoDbModel>(
        r#"
        SELECT 
            vi.id as vesting_id,
            vi.vesting_address,
            vi.timestamp_start,
            vi.timestamp_end,
            pi.id as premarket_id,
            pi.bc_address as premarket_address,
            pi.mint_address,
            pi.creator_id,
            pi.creator_address,
            vi.vesting_period,
            vi.init_unlock,
            pi.name,
            pi.symbol
        FROM vesting_info vi
        INNER JOIN premarket_info pi ON vi.premarket_id = pi.id
        WHERE pi.id = $1
        "#
    )
    .bind(premarket_id)
    .fetch_optional(pool)
    .await
}

/// Get vesting info by vesting_address
pub async fn get_vesting_info_by_vesting_address(
    pool: &PgPool,
    vesting_address: &str,
) -> Result<Option<FullVestingInfoDbModel>, sqlx::Error> {
    sqlx::query_as::<_, FullVestingInfoDbModel>(
        r#"
        SELECT 
            vi.id as vesting_id,
            vi.vesting_address,
            vi.timestamp_start,
            vi.timestamp_end,
            pi.id as premarket_id,
            pi.bc_address as premarket_address,
            pi.mint_address,
            pi.creator_id,
            pi.creator_address,
            vi.vesting_period,
            vi.init_unlock,
            pi.name,
            pi.symbol
        FROM vesting_info vi
        INNER JOIN premarket_info pi ON vi.premarket_id = pi.id
        WHERE vi.vesting_address = $1
        "#
    )
    .bind(vesting_address)
    .fetch_optional(pool)
    .await
}

/// Get vesting info by mint_address
pub async fn get_vesting_info_by_mint_address(
    pool: &PgPool,
    mint_address: &str,
) -> Result<Option<FullVestingInfoDbModel>, sqlx::Error> {
    sqlx::query_as::<_, FullVestingInfoDbModel>(
        r#"
        SELECT 
            vi.id as vesting_id,
            vi.vesting_address,
            vi.timestamp_start,
            vi.timestamp_end,
            pi.id as premarket_id,
            pi.bc_address as premarket_address,
            pi.mint_address,
            pi.creator_id,
            pi.creator_address,
            vi.vesting_period,
            vi.init_unlock,
            pi.name,
            pi.symbol
        FROM vesting_info vi
        INNER JOIN premarket_info pi ON vi.premarket_id = pi.id
        WHERE pi.mint_address = $1
        LIMIT 1
        "#
    )
    .bind(mint_address)
    .fetch_optional(pool)
    .await
}

/// Get vesting info by premarket bc_address
pub async fn get_vesting_info_by_premarket_address(
    pool: &PgPool,
    premarket_address: &str,
) -> Result<Option<FullVestingInfoDbModel>, sqlx::Error> {
    sqlx::query_as::<_, FullVestingInfoDbModel>(
        r#"
        SELECT 
            vi.id as vesting_id,
            vi.vesting_address,
            vi.timestamp_start,
            vi.timestamp_end,
            pi.id as premarket_id,
            pi.bc_address as premarket_address,
            pi.mint_address,
            pi.creator_id,
            pi.creator_address,
            vi.vesting_period,
            vi.init_unlock,
            pi.name,
            pi.symbol
        FROM vesting_info vi
        INNER JOIN premarket_info pi ON vi.premarket_id = pi.id
        WHERE pi.bc_address = $1
        LIMIT 1
        "#
    )
    .bind(premarket_address)
    .fetch_optional(pool)
    .await
}

/// Get all vesting holders (from premarket_holders) for a premarket_id with user information
pub async fn get_vesting_holders_by_premarket_id(
    pool: &PgPool,
    premarket_id: Uuid,
) -> Result<Vec<HolderDbModel>, sqlx::Error> {
    sqlx::query_as::<_, HolderDbModel>(
        r#"
        SELECT 
            ph.id,
            ph.premarket_info_id,
            ph.holder_id,
            ph.holder_wallet,
            ph.amount_lamport,
            ph.join_timestamp,
            ph.out_timestamp,
            ph.claimed,
            u.username,
            u.avatar_url,
            ph.amount_token,
            ph.claimed_amount_token,
            ph.updated_at
        FROM premarket_holders ph
        LEFT JOIN users u ON ph.holder_id = u.id
        WHERE ph.premarket_info_id = $1
          AND ph.out_timestamp IS NULL
        ORDER BY ph.amount_token DESC NULLS LAST, ph.amount_lamport DESC
        "#
    )
    .bind(premarket_id)
    .fetch_all(pool)
    .await
}

/// Create vesting info record
/// Returns service model VestingInfo
pub async fn create_vesting_info(
    pool: &PgPool,
    premarket_id: Uuid,
    vesting_address: &str,
    vesting_period: i64,
    init_unlock: i64,
    timestamp_start: Option<i64>,
    timestamp_end: Option<i64>,
) -> Result<VestingInfo> {
    // Insert vesting info
    sqlx::query_as::<_, VestingInfoDbModel>(
        r#"
        INSERT INTO vesting_info (premarket_id, vesting_address, vesting_period, init_unlock, timestamp_start, timestamp_end)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, premarket_id, vesting_address, vesting_period, init_unlock, timestamp_start, timestamp_end, created_at, updated_at
        "#
    )
    .bind(premarket_id)
    .bind(vesting_address)
    .bind(vesting_period)
    .bind(init_unlock)
    .bind(timestamp_start)
    .bind(timestamp_end)
    .fetch_one(pool)
    .await
    .map_err(|e| anyhow::anyhow!("Failed to create vesting info: {}", e))?;

    // Fetch full vesting info with premarket data
    let full_vesting_db = get_vesting_info_by_premarket_id(pool, premarket_id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch vesting info: {}", e))?
        .ok_or_else(|| anyhow::anyhow!("Vesting info not found after creation"))?;

    // Convert to service model
    full_vesting_db.try_into()
}

/// Update vesting timestamps (when vesting starts)
pub async fn update_vesting_timestamps(
    pool: &PgPool,
    vesting_id: Uuid,
    timestamp_start: i64,
    timestamp_end: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE vesting_info 
        SET timestamp_start = $2, timestamp_end = $3, updated_at = now()
        WHERE id = $1
        "#
    )
    .bind(vesting_id)
    .bind(timestamp_start)
    .bind(timestamp_end)
    .execute(pool)
    .await?;
    Ok(())
}

/// Update holder's token amounts (amount_token and claimed_amount_token) in premarket_holders
pub async fn update_holder_token_amounts(
    pool: &PgPool,
    premarket_id: Uuid,
    holder_wallet: &str,
    amount_token: i64,
    claimed_amount_token: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE premarket_holders 
        SET amount_token = $3, claimed_amount_token = $4, updated_at = now()
        WHERE premarket_info_id = $1 AND holder_wallet = $2
        "#
    )
    .bind(premarket_id)
    .bind(holder_wallet)
    .bind(amount_token)
    .bind(claimed_amount_token)
    .execute(pool)
    .await?;
    Ok(())
}

/// Update tokens claimed for a holder
pub async fn update_tokens_claimed(
    pool: &PgPool,
    premarket_id: Uuid,
    holder_wallet: &str,
    claimed_amount_token: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE premarket_holders 
        SET claimed_amount_token = $3, updated_at = now()
        WHERE premarket_info_id = $1 AND holder_wallet = $2
        "#
    )
    .bind(premarket_id)
    .bind(holder_wallet)
    .bind(claimed_amount_token)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get holder by wallet from premarket_holders
pub async fn get_holder_by_wallet(
    pool: &PgPool,
    premarket_id: Uuid,
    holder_wallet: &str,
) -> Result<Option<HolderDbModel>, sqlx::Error> {
    sqlx::query_as::<_, HolderDbModel>(
        r#"
        SELECT 
            ph.id,
            ph.premarket_info_id,
            ph.holder_id,
            ph.holder_wallet,
            ph.amount_lamport,
            ph.join_timestamp,
            ph.out_timestamp,
            ph.claimed,
            u.username,
            u.avatar_url,
            ph.amount_token,
            ph.claimed_amount_token,
            ph.updated_at
        FROM premarket_holders ph
        LEFT JOIN users u ON ph.holder_id = u.id
        WHERE ph.premarket_info_id = $1 AND ph.holder_wallet = $2
        "#
    )
    .bind(premarket_id)
    .bind(holder_wallet)
    .fetch_optional(pool)
    .await
}
