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

    pub premarket_goal_pers: f64,
    pub premarket_goal_sol_lamp: i64,
    pub premarket_deadline: i64,
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
    pub avatar_url: Option<String>, // Just to response with join
    pub username: Option<String>,   // Just to response with join

}

pub struct HolderStats {
    pub holders: Vec<HolderDbModel>,
    pub total_active_count: i64,
    pub reserved_sol_lamp: i64,
    pub reserved_sol_24h_before_lamp: i64,
}