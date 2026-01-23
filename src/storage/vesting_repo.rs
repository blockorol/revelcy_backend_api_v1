use sqlx::PgPool;
use uuid::Uuid;

use super::models::{
    FullVestingInfoDbModel, VestingHolderDbModel, VestingHolderWithUserDbModel,
    VestingInfoDbModel,
};

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
        "#
    )
    .bind(premarket_address)
    .fetch_optional(pool)
    .await
}

/// Get all vesting holders for a vesting_info_id with user information
pub async fn get_vesting_holders_by_vesting_id(
    pool: &PgPool,
    vesting_info_id: Uuid,
) -> Result<Vec<VestingHolderWithUserDbModel>, sqlx::Error> {
    sqlx::query_as::<_, VestingHolderWithUserDbModel>(
        r#"
        SELECT 
            vh.id,
            vh.vesting_info_id,
            vh.holder_id,
            vh.holder_wallet,
            vh.tokens_total,
            vh.tokens_claimed,
            vh.created_at,
            vh.updated_at,
            u.username,
            u.avatar_url
        FROM vesting_holders vh
        LEFT JOIN users u ON vh.holder_id = u.id
        WHERE vh.vesting_info_id = $1
        ORDER BY vh.tokens_total DESC
        "#
    )
    .bind(vesting_info_id)
    .fetch_all(pool)
    .await
}

/// Create vesting info record
pub async fn create_vesting_info(
    pool: &PgPool,
    premarket_id: Uuid,
    vesting_address: &str,
    vesting_period: i64,
    init_unlock: i64,
    timestamp_start: Option<i64>,
    timestamp_end: Option<i64>,
) -> Result<VestingInfoDbModel, sqlx::Error> {
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

/// Create or update vesting holder
pub async fn upsert_vesting_holder(
    pool: &PgPool,
    vesting_info_id: Uuid,
    holder_wallet: &str,
    holder_id: Option<Uuid>,
    tokens_total: i64,
) -> Result<VestingHolderDbModel, sqlx::Error> {
    sqlx::query_as::<_, VestingHolderDbModel>(
        r#"
        INSERT INTO vesting_holders (vesting_info_id, holder_wallet, holder_id, tokens_total)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (vesting_info_id, holder_wallet) 
        DO UPDATE SET 
            tokens_total = $4,
            holder_id = COALESCE($3, vesting_holders.holder_id),
            updated_at = now()
        RETURNING id, vesting_info_id, holder_id, holder_wallet, tokens_total, tokens_claimed, created_at, updated_at
        "#
    )
    .bind(vesting_info_id)
    .bind(holder_wallet)
    .bind(holder_id)
    .bind(tokens_total)
    .fetch_one(pool)
    .await
}

/// Update tokens claimed for a holder
pub async fn update_tokens_claimed(
    pool: &PgPool,
    vesting_info_id: Uuid,
    holder_wallet: &str,
    tokens_claimed: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE vesting_holders 
        SET tokens_claimed = $3, updated_at = now()
        WHERE vesting_info_id = $1 AND holder_wallet = $2
        "#
    )
    .bind(vesting_info_id)
    .bind(holder_wallet)
    .bind(tokens_claimed)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get vesting holder by wallet
pub async fn get_vesting_holder_by_wallet(
    pool: &PgPool,
    vesting_info_id: Uuid,
    holder_wallet: &str,
) -> Result<Option<VestingHolderWithUserDbModel>, sqlx::Error> {
    sqlx::query_as::<_, VestingHolderWithUserDbModel>(
        r#"
        SELECT 
            vh.id,
            vh.vesting_info_id,
            vh.holder_id,
            vh.holder_wallet,
            vh.tokens_total,
            vh.tokens_claimed,
            vh.created_at,
            vh.updated_at,
            u.username,
            u.avatar_url
        FROM vesting_holders vh
        LEFT JOIN users u ON vh.holder_id = u.id
        WHERE vh.vesting_info_id = $1 AND vh.holder_wallet = $2
        "#
    )
    .bind(vesting_info_id)
    .bind(holder_wallet)
    .fetch_optional(pool)
    .await
}
