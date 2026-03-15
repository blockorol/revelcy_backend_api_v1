use anyhow::{Result, bail};
use std::str::FromStr;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user::User;
use crate::models::whitelist::{WhitelistStatus, WhitelistUserInfo};
use crate::storage::models::WhitelistDbModel;

#[derive(sqlx::FromRow)]
struct WhitelistUserRow {
    id: Uuid,
    username: Option<String>,
    avatar_url: Option<String>,
    wallets: Vec<String>,
    status: String,
}

#[derive(sqlx::FromRow)]
struct WhitelistStatusRow {
    status: String,
    updated_at: i64,
}

/// is user exist in whitelist
pub async fn exists(
    pool: &PgPool,
    premarket_id: Uuid,
    user_id: Uuid,
) -> Result<bool> {
    let v = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM whitelist
            WHERE premarket_id = $1 AND user_id = $2
        )
        "#,
    )
    .bind(premarket_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(v)
}

pub async fn get_status(
    pool: &PgPool,
    premarket_id: Uuid,
    user_id: Uuid,
) -> Result<Option<WhitelistStatus>> {
    let status_opt = sqlx::query_scalar::<_, String>(
        r#"
        SELECT status
        FROM whitelist
        WHERE premarket_id = $1
          AND user_id = $2
        "#,
    )
    .bind(premarket_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let status = match status_opt {
        Some(s) => Some(WhitelistStatus::from_str(&s)?),
        None => None,
    };

    Ok(status)
}


pub async fn list_users_by_premarket(
    pool: &PgPool,
    premarket_id: Uuid,
    cursor: i64,
    limit: i64,
) -> Result<(Vec<WhitelistUserInfo>, i64)> {
    let limit = limit.clamp(1, 200);
    let offset = cursor.max(0);

    let total: i64 = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM whitelist
        WHERE premarket_id = $1
        "#,
    )
    .bind(premarket_id)
    .fetch_one(pool)
    .await?;

    let rows: Vec<WhitelistUserRow> = sqlx::query_as::<_, WhitelistUserRow>(
        r#"
        SELECT
            u.id,
            u.username,
            u.avatar_url,
            w.status,
            COALESCE(
                array_agg(DISTINCT wa.wallet_address)
                    FILTER (WHERE wa.wallet_address IS NOT NULL),
                '{}'::text[]
            ) AS wallets
        FROM whitelist w
        JOIN users u ON u.id = w.user_id
        LEFT JOIN wallets wa ON wa.user_id = u.id
        WHERE w.premarket_id = $1
        GROUP BY w.id, w.status, u.id, u.username, u.avatar_url
        ORDER BY w.id DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(premarket_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let users: Vec<WhitelistUserInfo> = rows
        .into_iter()
        .map(|u| Ok(WhitelistUserInfo {
            user: User {
                id: u.id,
                username: u.username,
                avatar_url: u.avatar_url,
                wallets: u.wallets,
            },
            status: WhitelistStatus::from_str(&u.status)?,
        }))
        .collect::<Result<Vec<_>>>()?;

    Ok((users, total))
}

pub async fn get_status_with_updated_at(
    pool: &PgPool,
    premarket_id: Uuid,
    user_id: Uuid,
) -> Result<Option<(WhitelistStatus, i64)>> {
    let row_opt = sqlx::query_as::<_, WhitelistStatusRow>(
        r#"
        SELECT
            status,
            EXTRACT(EPOCH FROM updated_at)::BIGINT AS updated_at
        FROM whitelist
        WHERE premarket_id = $1
          AND user_id = $2
        "#,
    )
    .bind(premarket_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let status = match row_opt {
        Some(r) => Some((WhitelistStatus::from_str(&r.status)?, r.updated_at)),
        None => None,
    };

    Ok(status)
}

pub async fn list_users_by_status(
    pool: &PgPool,
    premarket_id: Uuid,
    status: WhitelistStatus,
    cursor: i64,
    limit: i64,
) -> Result<(Vec<WhitelistUserInfo>, i64)> {
    let limit = limit.clamp(1, 200);
    let offset = cursor.max(0);

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM whitelist
        WHERE premarket_id = $1
          AND status = $2
        "#,
    )
    .bind(premarket_id)
    .bind(status.as_str())
    .fetch_one(pool)
    .await?;

    let rows: Vec<WhitelistUserRow> = sqlx::query_as::<_, WhitelistUserRow>(
        r#"
        SELECT
            u.id,
            u.username,
            u.avatar_url,
            w.status,
            COALESCE(
                array_agg(DISTINCT wa.wallet_address)
                    FILTER (WHERE wa.wallet_address IS NOT NULL),
                '{}'::text[]
            ) AS wallets
        FROM whitelist w
        JOIN users u ON u.id = w.user_id
        LEFT JOIN wallets wa ON wa.user_id = u.id
        WHERE w.premarket_id = $1
          AND w.status = $2
        GROUP BY w.updated_at, w.status, u.id, u.username, u.avatar_url
        ORDER BY w.updated_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(premarket_id)
    .bind(status.as_str())
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let users: Vec<WhitelistUserInfo> = rows
        .into_iter()
        .map(|u| Ok(WhitelistUserInfo {
            user: User {
                id: u.id,
                username: u.username,
                avatar_url: u.avatar_url,
                wallets: u.wallets,
            },
            status: WhitelistStatus::from_str(&u.status)?,
        }))
        .collect::<Result<Vec<_>>>()?;

    Ok((users, total))
}


/// CREATE: add user to whitelist or get existing record
pub async fn add(
    pool: &PgPool,
    premarket_id: Uuid,
    user_id: Uuid,
    status: WhitelistStatus,
) -> Result<()> {
    let id = Uuid::new_v4();

    let _row = sqlx::query_as::<_, WhitelistDbModel>(
        r#"
        INSERT INTO whitelist (id, premarket_id, user_id, status)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (premarket_id, user_id)
        DO UPDATE SET
            status = EXCLUDED.status,
            updated_at = NOW()
        RETURNING id, premarket_id, user_id
        "#,
    )
    .bind(id)
    .bind(premarket_id)
    .bind(user_id)
    .bind(status.as_str())
    .fetch_one(pool)
    .await?;

    Ok(())
}

/// CREATE: add many users to whitelist
pub async fn add_many(
    pool: &PgPool,
    premarket_id: Uuid,
    user_ids: &[Uuid],
    status: WhitelistStatus,
) -> Result<()> {
    if user_ids.is_empty() {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO whitelist (id, premarket_id, user_id, status)
        SELECT gen_random_uuid(), $1, x.user_id, $3
        FROM UNNEST($2::uuid[]) AS x(user_id)
        ON CONFLICT (premarket_id, user_id)
        DO UPDATE SET
            status = EXCLUDED.status,
            updated_at = NOW()
        "#,
    )
    .bind(premarket_id)
    .bind(user_ids)
    .bind(status.as_str())
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn add_requested_if_absent(
    pool: &PgPool,
    premarket_id: Uuid,
    user_id: Uuid,
) -> Result<bool> {
    let res = sqlx::query(
        r#"
        INSERT INTO whitelist (id, premarket_id, user_id, status)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (premarket_id, user_id) DO NOTHING
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(premarket_id)
    .bind(user_id)
    .bind(WhitelistStatus::Requested.as_str())
    .execute(pool)
    .await?;

    Ok(res.rows_affected() > 0)
}

/// DELETE: удалить по id
pub async fn delete_by_id(pool: &PgPool, id: Uuid) -> Result<u64> {
    let res = sqlx::query(r#"DELETE FROM whitelist WHERE id = $1"#)
        .bind(id)
        .execute(pool)
        .await?;

    let affected = res.rows_affected();
    if affected == 0 {
        bail!("whitelist row not found");
    }

    Ok(affected)
}

pub async fn delete_by_premarket_user(
    pool: &PgPool,
    premarket_id: Uuid,
    user_id: Uuid,
) -> Result<u64> {
    let res = sqlx::query(
        r#"
        DELETE FROM whitelist
        WHERE premarket_id = $1 AND user_id = $2
        "#,
    )
    .bind(premarket_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(res.rows_affected())
}

pub async fn update_status(
    pool: &PgPool,
    premarket_id: Uuid,
    user_id: Uuid,
    status: WhitelistStatus,
) -> Result<()> {
    let res = sqlx::query(
        r#"
        UPDATE whitelist
        SET status = $1,
            updated_at = NOW()
        WHERE premarket_id = $2
          AND user_id = $3
        "#,
    )
    .bind(status.as_str())
    .bind(premarket_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    if res.rows_affected() == 0 {
        bail!("whitelist row not found");
    }

    Ok(())
}
