use crate::models::user::UserFingerprintEventInsert;
use sqlx::{PgPool, Result};

pub async fn insert_user_fingerprint_event(
    pool: &PgPool,
    e: &UserFingerprintEventInsert,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO user_fingerprint_events (
            id,
            user_id,
            event_type,

            client_ts_ms,
            server_ts_ms,

            premarket,

            install_id,
            install_id_source,

            ip,
            user_agent,
            accept_language,

            sec_ch_ua,
            sec_ch_ua_platform,
            sec_ch_ua_mobile,

            client
        ) VALUES (
            $1,  $2,  $3,
            $4,  $5,
            $6,
            $7,  $8,
            $9,  $10, $11,
            $12, $13, $14,
            $15
        )
        "#,
    )
    .bind(e.id)
    .bind(e.user_id)
    .bind(&e.event_type)
    .bind(e.client_ts_ms)
    .bind(e.server_ts_ms)
    .bind(e.premarket.as_deref())
    .bind(&e.install_id)
    .bind(&e.install_id_source)
    .bind(&e.ip)
    .bind(&e.user_agent)
    .bind(&e.accept_language)
    .bind(&e.sec_ch_ua)
    .bind(&e.sec_ch_ua_platform)
    .bind(&e.sec_ch_ua_mobile)
    .bind(&e.client)
    .execute(pool)
    .await?;

    Ok(())
}
