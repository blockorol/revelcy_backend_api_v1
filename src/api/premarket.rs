use serde::{Deserialize, Serialize};
use uuid::Uuid;

// todo: unlock it and change network to that
// #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
// pub enum SolanaNetwork {
//     Devnet,
//     MainnetBeta,
// }
// impl<'de> Deserialize<'de> for SolanaNetwork {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let s: String = Deserialize::deserialize(deserializer)?;
//         match s.to_lowercase().as_str() {
//             "devnet"  => Ok(SolanaNetwork::Devnet),
//             "mainnet-beta" | "mainnet_beta" | "mainnet" | "mainnetbeta" => Ok(SolanaNetwork::MainnetBeta),
//             other => Err(serde::de::Error::custom(format!("unknown network: {}", other))),
//         }
//     }
// }

#[derive(Deserialize)]
pub struct GetMainInfoQuery {
    pub premarket_id: String,
}

#[derive(Serialize)]
pub struct GetMainInfoDTO {
    pub blockchain_info: BlockchainInfoDTO,
    pub community_info: CommunityInfoDTO,
}

#[derive(Deserialize)]
pub struct GetListQuery {
    pub cursor: u32,
    pub limit: u32,
}

#[derive(Serialize)]
pub struct GetListMainInfoDTO {
    pub premarkets: Option<Vec<BlockchainInfoDTO>>,
    pub total: Option<i64>,
}

#[derive(Deserialize)]
pub struct UpdateCommunityDTO {
    pub premarket_pubkey: String,
    pub community_info: CommunityInfoDTO,
}
#[derive(Deserialize)]
pub struct CreatePremarketDTO {
    pub blockchain_info: BlockchainInfoDTO,
    pub community_info: CommunityInfoDTO,
}

#[derive(Serialize, Deserialize)]
pub struct BlockchainInfoDTO {
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    pub symbol: String,
    pub image_url: Option<String>,
    pub ipfs_uri: String,
    pub creator_id:String,
    pub creator_address:String,
    pub premarket_address: String,
    pub links: TokenLinksDTO,
    pub premarket_goal_pers: f64,
    pub premarket_goal_sol_lamp: String,
    pub premarket_deadline: i64,
    pub premarket_created: i64,
    pub premarket_finished: Option<i64>,
    pub mint_address: String,
    pub state: TokenState,
}

#[derive(Serialize, Deserialize)]
pub struct TokenLinksDTO {
    pub telegram: Option<String>,
    pub twitter: Option<String>,
    pub web_site: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CommunityInfoDTO {
    pub description: String,
    pub token_banner_url: Option<String>,
    pub links: Option<Vec<CommunityLinkDTO>>,
}

#[derive(Serialize, Deserialize)]
pub struct CommunityLinkDTO {
    pub text: String,
    pub url: String,
    pub r#type: LinkTypeDTO,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkTypeDTO {
    X,
    Tg,
    Other,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenState {
    Premarket,
    Canceled,
    Finished,
}


#[derive(Deserialize)]
pub struct GetDynamicInfoQuery {
    pub premarket_id: String,
}

#[derive(Deserialize)]
pub struct GetHolderEntryPriceQuery {
    pub premarket_id: String,
    pub holder_wallet: String,
}

#[derive(Serialize)]
pub struct HolderEntryPriceDTO {
    #[serde(with = "string_as_number")]
    pub entry_price_lamp: f64,
}


#[derive(Serialize, Deserialize)]
pub struct TokenDynamicInfoDTO {
    pub holders_count: u32,

    #[serde(with = "string_as_number")]
    pub current_price_lamp: f64,

    #[serde(with = "string_as_number")]
    pub reserved_sol_lamp: u64,

    pub change_24h: f64,
    pub holders: Vec<HolderInfoDTO>,
}

#[derive(Serialize, Deserialize)]
pub struct HolderInfoDTO {
    pub id: Option<Uuid>,
    pub wallet_address: String,
    pub join_timestamp: i64,
    pub icon_url: Option<String>, // just to get based on user info
    pub username: Option<String>, // just to get based on user info
    #[serde(with = "string_as_number")]
    pub amount_sol_lamp: u64,
}

#[derive(serde::Deserialize)]
pub struct JoinPremarketTxRequest {
    pub network: String,
    pub user_pubkey: String,
    pub premarket_account: String,
    #[serde(with = "string_as_number")]
    pub amount_sol_lamp: u64,
}

#[derive(serde::Deserialize)]
pub struct OutPremarketTxRequest {
    pub network: String,
    pub user_pubkey: String,
    pub premarket_account: String,
}

#[derive(serde::Deserialize)]
pub struct FinishPremarketTxRequest {
    pub network: String,          // "devnet" | "mainnet-beta"
    pub user_pubkey: String,      // base58
    pub premarket_account: String // base58
}

#[derive(serde::Deserialize)]

pub struct DistributeTokensRequest {
    pub network: String,          // "devnet" | "mainnet-beta"
    pub user_pubkey: String,      // base58
    pub premarket_account: String, // base58
    pub token_mint: String,       // base58
    pub users: Vec<String>,        // base58
}

#[derive(serde::Deserialize)]
pub struct KillPremarketTxRequest {
    pub network: String,          // "devnet" | "mainnet-beta"
    pub user_pubkey: String,      // base58
    pub premarket_account: String, // base58
}

#[derive(serde::Deserialize)]
pub struct ExtendPremarketTxRequest {
    pub network: String,          // "devnet" | "mainnet-beta"
    pub user_pubkey: String,      // base58
    pub premarket_account: String, // base58
    pub new_deadline: i64,        // unix timestamp
}


#[derive(serde::Serialize)]
pub struct TxOnlyResponse {
    pub transaction: String,
}

#[derive(Deserialize, Debug)]
pub struct PremarketTransactionDTO {
    #[serde(rename = "premarket_pub_key")]
    pub premarket_pub_key: String,

    #[serde(rename = "user_wallet")]
    pub user_wallet: String,

    #[serde(rename = "user_id")]
    pub user_id: Option<Uuid>,

    pub tx: String,
}

#[derive(Deserialize)]
pub struct CreatePremarketTxRequest {
    pub network: String,         // "devnet" | "mainnet-beta"
    pub user_pubkey: String,     // base58
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub deadline: i64,           // unix sec
    #[serde(with = "string_as_number")]
    pub goal_sol_lamp: u64,
    #[serde(with = "string_as_number")]
    pub max_sol_lamp: u64,
    #[serde(with = "string_as_number")]
    pub creator_allocate_lamp: u64,
}

#[derive(Serialize)]
pub struct CreatePremarketTxResponse {
    pub transaction: String,           // base64(serialized Transaction)
    pub premarket_account_pda: String, // base58
    pub mint_address: String,
}


#[derive(Deserialize, Debug)]
pub struct UserJoinedToPremarketDTO {
    #[serde(flatten)]
    pub base: PremarketTransactionDTO,

    #[serde(
        rename = "join_amount_in_sol_lamport",
        with = "string_as_number"
    )]
    pub join_amount_in_sol_lamport: u64,
}

#[derive(Deserialize)]
pub struct FinishedPremarketDTO {
    pub base: PremarketTransactionDTO,
    pub network: String,          // "devnet" | "mainnet-beta"
}

mod string_as_number {
    use serde::{self, Serializer, Deserializer, Deserialize}; // <--- добавлен Deserialize
    use std::fmt::Display;
    use std::str::FromStr;

    pub fn serialize<S, T>(x: &T, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Display,
    {
        s.serialize_str(&x.to_string())
    }

    pub fn deserialize<'de, D, T>(d: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStr,
        T::Err: Display,
    {
        let s = String::deserialize(d)?; // теперь всё ок
        s.parse::<T>().map_err(serde::de::Error::custom)
    }
}

#[derive(Deserialize)]
pub struct UpdatePremarketDataDTO {
    pub network: String,          // "devnet" | "mainnet-beta"
    pub user_pubkey: String,      // base58
    pub premarket_account: String, // base58
    pub end_timestamp: Option<i64>,
    pub end_timestamp_updated: Option<bool>,
    pub goal_sol: Option<u64>,
    pub max_sol: Option<u64>,
    pub mint: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub uri: Option<String>,
    pub creator: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct DeployTxDTO {
    pub network: String,          // "devnet" | "mainnet-beta"
    pub tx: String,
}

#[derive(serde::Deserialize)]
pub struct CheckTxDTO {
    pub network: String,          // "devnet" | "mainnet-beta"
    pub sig: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckTxResponse {
    pub user_pubkey: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub uri: Option<String>,
    pub deadline: Option<i64>,
    pub goal_sol_lamp: Option<u64>,
    pub max_sol_lamp: Option<u64>,
    pub creator_allocate_lamp: Option<u64>,
    pub premarket: Option<String>,
    pub lamports_in: Option<u64>,
}