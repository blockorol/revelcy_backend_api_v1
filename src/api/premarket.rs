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
    pub premarket_id: Option<String>,
    pub premarket_name: Option<String>,
    pub network: String,
}

#[derive(Serialize)]
pub struct GetMainInfoDTO {
    pub blockchain_info: BlockchainInfoDTO,
    pub community_info: CommunityInfoDTO,
    pub availability_info: AvailabilityInfoDTO,
}

#[derive(Deserialize)]
pub struct GetListQuery {
    pub network: String,
    pub cursor: u32,
    pub limit: u32,
}

#[derive(Serialize)]
pub struct ShortPremarketInfoDTO {
    pub blockchain_info: BlockchainInfoDTO,
    pub availability_info: AvailabilityInfoDTO,
}

#[derive(Serialize)]
pub struct GetListMainInfoDTO {
    pub premarkets: Option<Vec<ShortPremarketInfoDTO>>,
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
    pub premarket_goal_sol_lamp: String,
    pub premarket_deadline: i64,
    pub premarket_is_extended: Option<bool>,
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

#[derive(Deserialize)]
pub struct UpdateAvailabilityInfoDTO {
    pub premarket_pubkey: String,
    pub token_short_url_name: Option<String>,
    pub is_hided: Option<bool>,
    pub network: String,
}

#[derive(Serialize, Deserialize)]
pub struct AvailabilityInfoDTO {
    pub token_short_url_name: Option<String>,
    pub is_hided: bool,
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
    pub claimed: bool,
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
    pub premarket_account: String, // base58
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
pub struct UpdateURITxRequest {
    pub network: String,          // "devnet" | "mainnet-beta"
    pub user_pubkey: String,      // base58
    pub premarket_account: String, // base58
    pub new_uri: String,        // new uri
}

#[derive(serde::Deserialize)]
pub struct ExtendPremarketTxRequest {
    pub network: String,          // "devnet" | "mainnet-beta"
    pub user_pubkey: String,      // base58
    pub premarket_account: String, // base58
    pub new_deadline: i64,        // unix timestamp
}

#[derive(serde::Deserialize)]
pub struct ClaimTokensTxRequest {
    pub network: String,          // "devnet" | "mainnet-beta"
    pub user_pubkey: String,      // base58
    pub premarket_account: String, // base58
    pub token_mint: String,       // base58
}

#[derive(serde::Deserialize)]
pub struct WithdrawVestingTxRequest {
    pub network: String,          // "devnet" | "mainnet-beta"
    pub user_pubkey: String,      // base58
    pub token_mint: String,       // base58
}

#[derive(serde::Deserialize)]
pub struct TokenClaimedDTO {
    pub network: String,          // "devnet" | "mainnet-beta"
    pub user_pubkey: String,      // base58
    pub premarket_account: String, // base58
}

#[derive(serde::Serialize)]
pub struct TokenClaimedResponse {
    pub claimed: bool,
    pub updated_in_db: bool,
}


#[derive(serde::Serialize)]
pub struct TxOnlyResponse {
    pub transaction: String,
}


#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Finalized,
    Failed,
}

#[derive(serde::Serialize)]
pub struct SentTxResponse  {
    pub signature: String,
    pub status: TransactionStatus,
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
pub struct TxToSignRequest {
    pub network: String,          // "devnet" | "mainnet-beta"
    pub unsigned_tx: String,      // base64(serialized Transaction)
    pub tx_type: String,          // "create_premarket" | "join_premarket" | ...
    pub premarket: Option<String>,
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

#[derive(Deserialize)]
pub struct ExtendedPremarketDTO {
    pub base: PremarketTransactionDTO,
    pub network: String,          // "devnet" | "mainnet-beta"
    pub new_deadline: i64,       // unix timestamp
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

// Vesting API DTOs
#[derive(Deserialize)]
pub struct GetVestingInfoQuery {
    pub lookup_type: String, // "premarket_id" | "premarket_address" | "vesting_address" | "mint_address"
    pub key: String,
}

#[derive(Serialize)]
pub struct VestingInfoDTO {
    pub vesting_id: String,
    pub vesting_address: String,
    pub premarket_id: String,
    pub premarket_address: String,
    #[serde(with = "string_as_number")]
    pub vesting_period: i64,
    #[serde(with = "string_as_number")]
    pub init_unlock: i64,
    pub timestamp_start: Option<i64>,
    pub timestamp_end: Option<i64>,
    pub is_active: bool,
}

#[derive(Serialize)]
pub struct VestingHolderDTO {
    pub holder_id: Option<String>,
    pub holder_wallet: String,
    #[serde(with = "string_as_number")]
    pub amount_sol_lamp: i64,
    pub amount_tokens: Option<i64>,
    pub claimed_tokens: Option<i64>,
    pub available_tokens: Option<i64>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Serialize)]
pub struct FullVestingInfoDTO {
    pub vesting_info: VestingInfoDTO,
    pub holders: Vec<VestingHolderDTO>,
    pub total_holders: usize,
    #[serde(with = "string_as_number")]
    pub total_tokens: i64,
    #[serde(with = "string_as_number")]
    pub total_tokens_claimed: i64,
}