use serde::{Serialize, Deserialize};
use serde_json::Value;

use solana_sdk::pubkey::Pubkey;
use uuid::Uuid;

#[derive(Debug)]
pub enum ApplyInviteCodeResult {
    Applied,
    InviteCodeNotFound,
    AlreadyApplied,
}


#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub wallets: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UserContextData {
    pub internal_id: Uuid,
    pub current_pubkey: Pubkey,
    pub wallets: Vec<Pubkey>,
}

pub struct Session {
    pub nonce: String,
}

pub struct WalletAddress {
    pub address: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserFingerprintEventInsert {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub event_type: String,

    pub client_ts_ms: Option<i64>,
    pub server_ts_ms: i64,

    pub premarket: Option<String>,

    pub install_id: String,
    pub install_id_source: String,

    pub ip: String,
    pub user_agent: String,
    pub accept_language: String,

    pub sec_ch_ua: String,
    pub sec_ch_ua_platform: String,
    pub sec_ch_ua_mobile: String,

    /// raw client json (ClientContext)
    pub client: Value,
}

#[derive(Serialize, Deserialize)]
pub struct ScreenInfo {
    pub height: Option<i32>,
    pub width: Option<i32>,
}

#[derive(Serialize, Deserialize)]
pub struct UserFingerprintEventFrontendData {
    pub user_id: Option<String>,
    pub event_type: String,
    
    pub install_id: String,
    pub install_id_source: String,

    pub client_ts_ms: Option<i64>,

    pub timezone: Option<String>,
    pub locale: Option<String>,
    pub languages: Option<Vec<String>>,
    pub language: Option<String>,
    pub screen: ScreenInfo,
    pub pixel_ratio: Option<f32>,

    pub user_agent: Option<String>,
    pub phantom_version: Option<String>,

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserFingerprintEventBackendData {
    pub ip: String,
    pub user_agent: String,
    pub accept_language: String,
    pub sec_ch_ua: String,
    pub sec_ch_ua_platform: String,

    pub sec_ch_ua_mobile: String,
}
