use sqlx::{PgPool, Result};
use uuid::Uuid;

pub async fn add_users(
    pool: &PgPool,
    premarket_id: Uuid,
    user_ids: &[Uuid],
) -> Result<u64> {
    if user_ids.is_empty() {
        return Ok(0);
    }

    let res = sqlx::query(
        r#"
        INSERT INTO whitelist (premarket_id, user_id)
        SELECT $1, x
        FROM UNNEST($2::uuid[]) AS t(x)
        ON CONFLICT (premarket_id, user_id) DO NOTHING
        "#,
    )
    .bind(premarket_id)
    .bind(user_ids) // &[Uuid] -> uuid[]
    .execute(pool)
    .await?;

    Ok(res.rows_affected())
}

/// Добавляет в whitelist по списку кошельков.
/// Кошельки, которых нет в таблице `wallets`, игнорируются.
/// Требуется UNIQUE INDEX на (premarket_id, user_id).
pub async fn add_wallets(
    pool: &PgPool,
    premarket_id: Uuid,
    wallet_addresses: &[String],
) -> Result<u64> {
    if wallet_addresses.is_empty() {
        return Ok(0);
    }

    let res = sqlx::query(
        r#"
        INSERT INTO whitelist (premarket_id, user_id)
        SELECT $1, w.user_id
        FROM wallets w
        WHERE w.wallet_address = ANY($2::text[])
        ON CONFLICT (premarket_id, user_id) DO NOTHING
        "#,
    )
    .bind(premarket_id)
    .bind(wallet_addresses) // &[String] -> text[]
    .execute(pool)
    .await?;

    Ok(res.rows_affected())
}


/// Проверяет доступ пользователя:
/// - true, если для данного bc_id в таблице нет ни одной записи (лист отсутствует)
/// - true, если пользователь есть в листе
/// - false, если лист есть, но данного пользователя в нём нет
pub async fn has_user_access(pool: &PgPool, bc_id: Uuid, user_id: Uuid) -> Result<bool> {
    let access: bool = sqlx::query_scalar(
        r#"
        SELECT
            CASE
                WHEN NOT EXISTS (
                    SELECT 1 FROM whitelist w WHERE w.premarket_id = $1
                ) THEN TRUE
                WHEN EXISTS (
                    SELECT 1 FROM whitelist w WHERE w.premarket_id = $1 AND w.user_id = $2
                ) THEN TRUE
                ELSE FALSE
            END AS access
        "#,
    )
    .bind(bc_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(access)
}

/// Возвращает список user_id из вайтлиста по premarket_id.
/// Если записей нет — вернёт пустой вектор.
pub async fn get_whitelist(pool: &PgPool, premarket_id: Uuid) -> Result<Vec<Uuid>> {
    let rows = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT user_id
        FROM whitelist
        WHERE premarket_id = $1
        ORDER BY user_id
        "#,
    )
    .bind(premarket_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
