use crate::storage::models::{SigningKeyPair};
use sqlx::{PgPool, Result};
use uuid::Uuid;

pub async fn insert_mint_signing_key(
    pool: &PgPool,
    premarket_pubkey: &str,
    pub_key: &str,
    priv_key: &str,
) -> Result<()> {

    sqlx::query(
        r#"
        INSERT INTO signing_keys (id, premarket_pubkey, pub_key, priv_key, type)
        VALUES ($1, $2, $3, $4, 'mint_key')
        ON CONFLICT (pub_key)
        DO UPDATE SET
            premarket_pubkey = EXCLUDED.premarket_pubkey,
            priv_key        = EXCLUDED.priv_key,
            type            = 'mint_key'
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(premarket_pubkey)
    .bind(pub_key)
    .bind(priv_key)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_signing_key_by_pubkey(
    pool: &PgPool,
    pub_key: &str,
) -> Result<u64> {
    let res = sqlx::query(
        r#"DELETE FROM signing_keys WHERE pub_key = $1"#,
    )
    .bind(pub_key)
    .execute(pool)
    .await?;

    Ok(res.rows_affected())
}


pub async fn get_mint_signing_keypair_by_premarket(
    pool: &PgPool,
    premarket_pubkey: &str,
) -> Result<Option<SigningKeyPair>> {
    let row = sqlx::query_as::<_, SigningKeyPair>(
        r#"
        SELECT pub_key, priv_key
        FROM signing_keys
        WHERE premarket_pubkey = $1
          AND type = 'mint_key'
        LIMIT 1
        "#,
    )
    .bind(premarket_pubkey)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn get_unused_signing_key(pool: &PgPool) -> Result<Option<SigningKeyPair>> {
    let row = sqlx::query_as::<_, SigningKeyPair>(
        r#"
        SELECT pub_key, priv_key
        FROM signing_keys
        ORDER BY 
            CASE WHEN premarket_pubkey = 'does_not_exist' THEN 0 ELSE 1 END,
            id
        LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn acquire_signing_key(
    pool: &PgPool,
    premarket_uuid: &str,
) -> Result<Option<SigningKeyPair>> {
    let key = sqlx::query_as::<_, SigningKeyPair>(
        r#"
        UPDATE signing_keys
        SET premarket_pubkey = $1
        WHERE id = (
            SELECT id
            FROM signing_keys
            WHERE premarket_pubkey = 'does_not_exist'
            ORDER BY id
            FOR UPDATE SKIP LOCKED
            LIMIT 1
        )
        RETURNING pub_key, priv_key
        "#
    )
    .bind(premarket_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(key)
}

pub async fn update_premarket_pubkey(
    pool: &PgPool,
    premarket_pubkey: &str,
    premarket_uuid: &Uuid
) -> Result<u64> {
    let res = sqlx::query(
        r#"
        UPDATE signing_keys
        SET premarket_pubkey = $1
        WHERE id = $2
        "#
    )
    .bind(premarket_pubkey)
    .bind(premarket_uuid)
    .execute(pool)
    .await?;


    Ok(res.rows_affected())
}
