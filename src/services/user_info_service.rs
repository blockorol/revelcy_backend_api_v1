use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use serde_json::{json, Value};

use crate::api::errors::{ApiError, ApiResult};
use crate::models::user::{
    UserFingerprintEventInsert,
    UserFingerprintEventFrontendData,
    UserFingerprintEventBackendData,
};
use crate::storage::user_info_repo;

pub async fn write_user_fingerprint_event(
    pool: &PgPool,
    user_id: Option<Uuid>,
    premarket: Option<String>,
    frontend_related_data: UserFingerprintEventFrontendData,
    backend_related_data: UserFingerprintEventBackendData,
) -> ApiResult<()> {
    let insert: UserFingerprintEventInsert = FingerprintEventParts {
        fe: frontend_related_data,
        be: backend_related_data,
        user_id,
        premarket,
    }
    .try_into()?;

    user_info_repo::insert_user_fingerprint_event(pool, &insert)
        .await
        .map_err(|e| {
            eprintln!("write_user_fingerprint_event: db error: {e:?}");
            ApiError::internal_server_error()
        })?;

    Ok(())
}

pub struct FingerprintEventParts {
    pub fe: UserFingerprintEventFrontendData,
    pub be: UserFingerprintEventBackendData,
    pub user_id: Option<Uuid>,
    pub premarket: Option<String>,
}

impl FingerprintEventParts {
    fn default_str(s: String) -> String {
        if s.trim().is_empty() { "default".to_string() } else { s }
    }

    fn default_opt_str(s: Option<String>) -> String {
        match s {
            Some(v) if !v.trim().is_empty() => v,
            _ => "default".to_string(),
        }
    }
}

impl TryFrom<FingerprintEventParts> for UserFingerprintEventInsert {
    type Error = ApiError;

    fn try_from(p: FingerprintEventParts) -> Result<Self, Self::Error> {
        let server_ts_ms = Utc::now().timestamp_millis();

        // raw client json (в client складывай то, что реально есть во FrontendData)
        let client: Value = json!({
            "user_id": p.fe.user_id,
            "timezone": p.fe.timezone,
            "locale": p.fe.locale,
            "language": p.fe.language,
            "languages": p.fe.languages,
            "screen": p.fe.screen,
            "pixelRatio": p.fe.pixel_ratio,
            "userAgent": p.fe.user_agent,
            "phantom_version": p.fe.phantom_version,
        });

        Ok(UserFingerprintEventInsert {
            id: Uuid::new_v4(),
            user_id: p.user_id, // <-- IMPORTANT: user_id должен быть Option<Uuid> в модели
            event_type: p.fe.event_type.to_string(), // если это enum — ниже покажу как сделать to_string()

            client_ts_ms: p.fe.client_ts_ms,
            server_ts_ms,

            premarket: p.premarket,

            install_id: FingerprintEventParts::default_str(p.fe.install_id),
            install_id_source: FingerprintEventParts::default_str(p.fe.install_id_source),

            ip: FingerprintEventParts::default_str(p.be.ip),
            user_agent: FingerprintEventParts::default_str(p.be.user_agent),
            accept_language: FingerprintEventParts::default_str(p.be.accept_language),

            sec_ch_ua: FingerprintEventParts::default_str(p.be.sec_ch_ua),
            sec_ch_ua_platform: FingerprintEventParts::default_str(p.be.sec_ch_ua_platform),
            sec_ch_ua_mobile: FingerprintEventParts::default_str(p.be.sec_ch_ua_mobile),

            client,
        })
    }
}
