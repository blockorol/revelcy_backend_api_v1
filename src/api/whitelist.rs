use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::premarket::Network;

#[derive(Debug, Clone, Deserialize)]
pub struct RemoveWhitelistUserRequest {
    pub network: Network,
    pub premarket_id: Uuid,
    pub user_id: Option<Uuid>,
    pub user_pubkey: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RemoveWhitelistUserResponse {
    pub removed: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddWhitelistUserRequest {
    pub network: Network,
    pub premarket_id: Uuid,

    // one of these required
    pub user_id: Option<Uuid>,
    pub user_pubkey: Option<String>, 
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddWhitelistUserListRequest {
    pub network: Network,
    pub premarket_id: Uuid,

    // можно передать любое или оба
    pub user_ids: Option<Vec<Uuid>>,
    pub user_pubkeys: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetWhitelistRequest {
    pub network: Network,
    pub premarket_id: Uuid,
    pub status: Option<String>,
    pub cursor: i64,
    pub limit: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistUserDTO {
    pub id: Uuid,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub wallets: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetWhitelistResponse {
    pub items: Vec<WhitelistUserDTO>,
    pub total: Option<i64>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistSetStatusRequest {
    pub network: Network,
    pub premarket_id: Uuid,

    pub user_id: Option<Uuid>,
    pub user_pubkey: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistSetStatusResponse {
    pub updated: u64,
}
