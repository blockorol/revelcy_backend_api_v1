use crate::storage::models::{SigningKeyPair, SigningKeyPairWithId};
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

// acquire_signing_key with uuid instead of premarket_pubkey
pub async fn acquire_signing_key(
    pool: &PgPool,
    premarket_uuid: &str,
) -> Result<Option<SigningKeyPairWithId>> {
    let key = sqlx::query_as::<_, SigningKeyPairWithId>(
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
        RETURNING id, pub_key, priv_key
        "#
    )
    .bind(premarket_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(key)
}

// second part of acquire_signing_key flow - update premarket_pubkey for the premarket pubkey key
pub async fn update_premarket_pubkey(
    pool: &PgPool,
    premarket_pubkey: &str,
    keypair_uuid: &Uuid
) -> Result<u64> {
    let res = sqlx::query(
        r#"
        UPDATE signing_keys
        SET premarket_pubkey = $1
        WHERE id = $2
        "#
    )
    .bind(premarket_pubkey)
    .bind(keypair_uuid)
    .execute(pool)
    .await?;


    Ok(res.rows_affected())
}
