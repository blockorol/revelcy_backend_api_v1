use crate::storage::models::UserDbModel;
use crate::models::user::User;
use sqlx::{PgPool, Row, Result};
use uuid::Uuid;

pub async fn get_users_by_ids(pool: &PgPool, user_ids: &[Uuid]) -> Result<Vec<User>> {
    if user_ids.is_empty() {
        return Ok(vec![]);
    }

    let rows = sqlx::query(
        r#"
        SELECT
            u.id,
            u.username,
            u.avatar_url,
            COALESCE(
              array_agg(w.wallet_address) FILTER (WHERE w.wallet_address IS NOT NULL),
              ARRAY[]::text[]
            ) AS wallets
        FROM users u
        LEFT JOIN wallets w ON w.user_id = u.id
        WHERE u.id = ANY($1::uuid[])
        GROUP BY u.id, u.username, u.avatar_url
        ORDER BY u.id
        "#,
    )
    .bind(user_ids)
    .fetch_all(pool)
    .await?;

    let users = rows
        .into_iter()
        .map(|row| User {
            id: row.get::<Uuid, _>("id"),
            username: row.try_get::<Option<String>, _>("username").ok().flatten(),
            avatar_url: row.try_get::<Option<String>, _>("avatar_url").ok().flatten(),
            wallets: row.get::<Vec<String>, _>("wallets"),
        })
        .collect();

    Ok(users)
}

// Найти пользователя по кошельку
pub async fn get_user_by_wallet(pool: &PgPool, wallet_address: &str) -> Result<Option<User>> {
    let record = sqlx::query_as::<_, UserDbModel>(
        r#"
        SELECT u.id, u.username, u.avatar_url
        FROM users u
        JOIN wallets w ON w.user_id = u.id
        WHERE w.wallet_address = $1
        "#
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
            "#
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


// Создать пользователя и привязать кошелек
pub async fn create_user_with_wallet(pool: &PgPool, wallet_address: &str) -> Result<User> {
    let mut tx = pool.begin().await?;

    let user_db = sqlx::query_as::<_, UserDbModel>(
        r#"
        INSERT INTO users (username, avatar_url)
        VALUES (NULL, NULL)
        RETURNING id, username, avatar_url
        "#
    )
    .fetch_one(&mut tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO wallets (wallet_address, user_id)
        VALUES ($1, $2)
        "#
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

pub async fn update_username(pool: &PgPool, user_id: Uuid, new_username: &str) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE users
        SET username = $1
        WHERE id = $2
        "#
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
        "#
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
        "#
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        DELETE FROM users
        WHERE id = $1
        "#
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}
