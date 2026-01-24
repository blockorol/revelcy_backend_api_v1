use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct SigningKeyDbModel {
    pub id: Uuid,
    pub premarket_pubkey: String,
    pub pub_key: String,
    pub priv_key: String,
    pub r#type: String,
}

#[derive(FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct SigningKeyPair {
    pub pub_key: String,
    pub priv_key: String,
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct UserDbModel {
    pub id: Uuid,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct WalletDbModel {
    pub wallet_address: String,
    pub user_id: Uuid,
}



#[derive(sqlx::FromRow, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PremarketInfoDbModel {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub creator_address: String,
    pub bc_address: String,
    pub short_url_name: Option<String>,
    pub data_uri: String,
    pub mint_address: String,
    pub name: String,
    pub description: String,
    pub symbol: String,
    pub image_url: Option<String>,

    // Ссылки
    pub telegram: Option<String>,
    pub twitter: Option<String>,
    pub web_site: Option<String>,

    pub premarket_goal_sol_lamp: i64,
    pub premarket_deadline: i64,
    pub is_extended: bool,
    pub is_hided: bool,
    pub premarket_created: i64,
    pub premarket_finished: Option<i64>,

    pub state: String, // лучше использовать enum, но можно и строку
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommunityInfoDbModel {
    pub id: Uuid,                     // тот же ID, что и у PremarketInfo (foreign key)
    pub description: String,
    pub token_banner_url: Option<String>,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommunityLinkDbModel {
    pub id: Uuid,
    pub community_info_id: Uuid,   // внешний ключ на `CommunityInfoDbModel::id`
    pub text: String,
    pub url: String,
    pub r#type: String,            // 'x', 'tg', 'other'
}


#[derive(Debug, Clone, sqlx::FromRow)]
pub struct HolderDbModel {
    pub id: Uuid,
    pub premarket_info_id: Uuid,
    pub holder_id: Option<Uuid>,
    pub holder_wallet: String,
    pub amount_lamport: i64,
    pub join_timestamp: i64,
    pub out_timestamp: Option<i64>,
    pub claimed: bool,
    pub avatar_url: Option<String>, // Just to response with join
    pub username: Option<String>,   // Just to response with join
    // Vesting-related fields
    pub amount_token: Option<i64>,
    pub claimed_amount_token: Option<i64>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct HolderStats {
    pub holders: Vec<HolderDbModel>,
    pub total_active_count: i64,
    pub reserved_sol_lamp: i64,
    pub reserved_sol_24h_before_lamp: i64,
}

// Vesting Models
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct VestingInfoDbModel {
    pub id: Uuid,
    pub premarket_id: Uuid,
    pub vesting_address: String,
    pub vesting_period: i64,
    pub init_unlock: i64,
    pub timestamp_start: Option<i64>,
    pub timestamp_end: Option<i64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// Full vesting info with premarket data
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FullVestingInfoDbModel {
    // Vesting info fields
    pub vesting_id: Uuid,
    pub vesting_address: String,
    pub timestamp_start: Option<i64>,
    pub timestamp_end: Option<i64>,
    // Premarket fields needed for vesting
    pub premarket_id: Uuid,
    pub premarket_address: String,
    pub mint_address: String,
    pub creator_id: Uuid,
    pub creator_address: String,
    pub vesting_period: i64,
    pub init_unlock: i64,
    pub name: String,
    pub symbol: String,
}