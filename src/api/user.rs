use serde::{Deserialize, Serialize};


// ======= HTTP DTO User =======
#[derive(Debug, Deserialize)]
pub struct SetInviteCodeRequestDto {
    pub invite_code: String,
}

#[derive(Debug, Serialize)]
pub struct SetInviteCodeResponseDto {
    pub jwt: String,
}

#[derive(Deserialize)]
pub struct AddUserNameRequestDto {
    pub username: String,
}

#[derive(Serialize)]
pub struct AddUserNameResponseDto {
    pub username: String,
    pub jwt: String,
}

#[derive(Serialize)]
pub struct AddAvatarResponseDto {
    pub avatar_url: String,
    pub jwt: String,
}


// ======= HTTP DTO Additional info =======
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserInfoEventType {
    Register,
    Login,
    Logout,
    DailyActive,
    JoinPremarket,
    OutPremarket,
    ClaimToken,
    CreatePremarket,
    FinishPremarket,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientContextDTO {
    // from FE
    pub user_id: Option<String>,
    pub user_agent: Option<String>,
    pub language: Option<String>,       // navigator.language
    pub languages: Option<Vec<String>>, // navigator.languages
    pub timezone: Option<String>,       // Intl.DateTimeFormat().resolvedOptions().timeZone
    pub locale: Option<String>,         // e.g. "en-US"
    pub screen_width: Option<i32>,
    pub screen_height: Option<i32>,
    pub pixel_ratio: Option<f32>,

    // storage identifiers
    pub install_id: Option<String>,    // for RN/Expo: persisted id
    pub install_id_source: Option<String>,    // for RN/Expo: persisted id

    // phantom
    pub phantom_version: Option<String>, // if you can detect it
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSetInfoRequestDTO {
    /// Optional "reason"/event that caused this record
    pub event_type: UserInfoEventType,

    /// FE timestamp (ms since epoch) for debugging/ordering; BE should also set its own timestamp.
    pub client_timestamp_ms: Option<i64>,

    /// premaket etc (optional metadata)
    pub premarket: Option<String>,

    /// Collected context
    pub client: ClientContextDTO,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSetInfoResponseDTO {
    pub ok: bool,
}
