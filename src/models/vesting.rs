use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VestingSettingsServiceModel {
    pub enabled: bool,
    pub vesting_period_sec: Option<i64>,
    pub unlock_at_launch_percent: Option<i64>,
}

// Vesting Service Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VestingInfo {
    pub vesting_id: Uuid,
    pub vesting_address: String,
    pub premarket_id: Uuid,
    pub premarket_address: String,
    pub mint_address: String,
    pub creator_id: Uuid,
    pub creator_address: String,
    pub vesting_period: i64,
    pub init_unlock: i64,
    pub timestamp_start: Option<i64>,
    pub timestamp_end: Option<i64>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VestingHolderInfo {
    pub holder_id: Option<Uuid>,
    pub holder_wallet: String,
    pub amount_sol_lamp: i64,
    pub amount_tokens: Option<i64>,
    pub claimed_tokens: Option<i64>,
    pub available_tokens: Option<i64>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VestingHolderPosition {
    pub holder_id: Option<Uuid>,
    pub holder_wallet: String,
    pub amount_sol_lamp: i64,
    pub amount_token: i64,
    pub claimed_amount_token: i64,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullVestingInfo {
    pub vesting_info: VestingInfo,
    pub holders: Vec<VestingHolderInfo>,
    pub total_holders: usize,
    pub total_tokens: i64,
    pub total_tokens_claimed: i64,
}
