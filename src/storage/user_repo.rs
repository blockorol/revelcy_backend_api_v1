use crate::models::user::{ApplyInviteCodeResult, User, UserShort};
use crate::storage::models::UserDbModel;
use sqlx::{PgPool, Result, Row};
use uuid::Uuid;

pub async fn get_user_by_wallet(pool: &PgPool, wallet_address: &str) -> Result<Option<User>> {
    let record = sqlx::query_as::<_, UserDbModel>(
        r#"
        SELECT u.id, u.username, u.avatar_url
        FROM users u
        JOIN wallets w ON w.user_id = u.id
        WHERE w.wallet_address = $1
        "#,
    )
    .bind(wallet_address)
    .fetch_optional(pool)
    .await?;

    if let Some(user_db) = record {
        let wallets = sqlx::query(
            r#"
            SELECT wallet_address
            FROM wallets
            WHERE user_id = $1
            "#,
        )
        .bind(user_db.id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| row.get::<String, _>("wallet_address"))
        .collect();

        Ok(Some(User {
            id: user_db.id,
            username: user_db.username,
            avatar_url: user_db.avatar_url,
            wallets,
        }))
    } else {
        Ok(None)
    }
}

pub async fn search_users_by_username_with_wallets(
    pool: &PgPool,
    input: &str,
    limit: i64,
) -> Result<Vec<User>> {
    let limit = limit.clamp(1, 50);
    let pattern = format!("%{}%", input);

    let users_db = sqlx::query_as::<_, UserDbModel>(
        r#"
        SELECT id, username, avatar_url
        FROM users
        WHERE username ILIKE $1
        ORDER BY username
        LIMIT $2
        "#,
    )
    .bind(&pattern)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::with_capacity(users_db.len());

    for user_db in users_db {
        let wallets = sqlx::query(
            r#"
            SELECT wallet_address
            FROM wallets
            WHERE user_id = $1
            "#,
        )
        .bind(user_db.id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| row.get::<String, _>("wallet_address"))
        .collect();

        result.push(User {
            id: user_db.id,
            username: user_db.username,
            avatar_url: user_db.avatar_url,
            wallets,
        });
    }

    Ok(result)
}

pub async fn get_users_short_by_addresses(
    pool: &PgPool,
    addresses: &[String],
) -> Result<Vec<UserShort>> {
    let rows = sqlx::query(
        r#"
        SELECT
            w.wallet_address AS address,
            u.username AS name,
            u.avatar_url AS url
        FROM wallets w
        JOIN users u ON u.id = w.user_id
        WHERE w.wallet_address = ANY($1)
        ORDER BY w.wallet_address
        "#,
    )
    .bind(addresses)
    .fetch_all(pool)
    .await?;

    let items = rows
        .into_iter()
        .map(|row| UserShort {
            address: row.get::<String, _>("address"),
            name: row.get::<Option<String>, _>("name"),
            url: row.get::<Option<String>, _>("url"),
        })
        .collect();

    Ok(items)
}

pub async fn create_user_with_wallet(pool: &PgPool, wallet_address: &str) -> Result<User> {
    let mut tx = pool.begin().await?;

    let user_db = sqlx::query_as::<_, UserDbModel>(
        r#"
        INSERT INTO users (username, avatar_url, status)
        VALUES (NULL, NULL, 'INITIALISED')
        RETURNING id, username, avatar_url
        "#,
    )
    .fetch_one(&mut tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO wallets (wallet_address, user_id)
        VALUES ($1, $2)
        "#,
    )
    .bind(wallet_address)
    .bind(user_db.id)
    .execute(&mut tx)
    .await?;

    tx.commit().await?;

    Ok(User {
        id: user_db.id,
        username: user_db.username,
        avatar_url: user_db.avatar_url,
        wallets: vec![wallet_address.to_string()],
    })
}

pub async fn apply_invite_code_once(
    pool: &PgPool,
    user_id: Uuid,
    invite_code: &str,
) -> Result<ApplyInviteCodeResult> {
    let invite_code_id: Option<Uuid> = sqlx::query_scalar(
        r#"
        SELECT id
        FROM invite_codes
        WHERE code = $1 AND is_active = TRUE
        "#,
    )
    .bind(invite_code)
    .fetch_optional(pool)
    .await?;

    let Some(invite_code_id) = invite_code_id else {
        return Ok(ApplyInviteCodeResult::InviteCodeNotFound);
    };

    let res = sqlx::query(
        r#"
        INSERT INTO user_invites (user_id, invite_code_id)
        VALUES ($1, $2)
        ON CONFLICT (user_id) DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(invite_code_id)
    .execute(pool)
    .await?;

    if res.rows_affected() == 0 {
        return Ok(ApplyInviteCodeResult::AlreadyApplied);
    }

    Ok(ApplyInviteCodeResult::Applied)
}

pub async fn update_username(pool: &PgPool, user_id: Uuid, new_username: &str) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE users
        SET
            username = $1,
            status = 'REGISTERED'
        WHERE id = $2
        "#,
    )
    .bind(new_username)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_avatar_url(pool: &PgPool, user_id: Uuid, new_avatar_url: &str) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE users
        SET avatar_url = $1
        WHERE id = $2
        "#,
    )
    .bind(new_avatar_url)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_user_by_id(pool: &PgPool, user_id: Uuid) -> Result<()> {
    sqlx::query(
        r#"
        DELETE FROM wallets
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        DELETE FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}
