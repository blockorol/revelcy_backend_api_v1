use serde::{Deserialize, Serialize};

// --------- Update ------------------
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateVestingInfoRequest {
    pub user_pubkey: String,
    pub network: String,
    pub premarket_pubkey: String,
    pub vesting_period_sec: u64,
    pub unlock_at_launch_percent: u64,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateVestingInfoResponse {
    pub ok: bool,
}
