use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::models::premarket::{CommunityLink, LinkType};
use crate::api::vesting::VestingSettingsDTO;
use std::fmt;


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
    pub vesting_info: Option<VestingSettingsDTO>,
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
    pub is_whitelist_enabled: Option<bool>,
    pub network: String,
}

#[derive(Serialize, Deserialize)]
pub struct AvailabilityInfoDTO {
    pub token_short_url_name: Option<String>,
    pub is_hided: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommunityInfoDTO {
    pub description: String,
    pub token_banner_url: Option<String>,
    pub links: Option<Vec<CommunityLinkDTO>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommunityLinkDTO {
    pub text: String,
    pub url: String,
    pub r#type: LinkTypeDTO,
}
impl From<CommunityLinkDTO> for CommunityLink {
    fn from(v: CommunityLinkDTO) -> Self {
        Self {
            text: v.text,
            url: v.url,
            r#type: v.r#type.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum LinkTypeDTO {
    X,
    Tg,
    Other,
}
impl From<LinkTypeDTO> for LinkType {
    fn from(v: LinkTypeDTO) -> Self {
        match v {
            LinkTypeDTO::X => LinkType::X,
            LinkTypeDTO::Tg => LinkType::Tg,
            LinkTypeDTO::Other => LinkType::Other,
        }
    }
}


// From Service → DTO
impl From<LinkType> for LinkTypeDTO {
    fn from(value: LinkType) -> Self {
        match value {
            LinkType::X => LinkTypeDTO::X,
            LinkType::Tg => LinkTypeDTO::Tg,
            LinkType::Other => LinkTypeDTO::Other,
        }
    }
}


#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenState {
    Concept,
    Premarket,
    Canceled,
    Finished,
}


#[derive(Deserialize)]
pub struct GetHolderEntryInfoQuery {
    pub premarket_id: String,
    pub holder_wallet: String,
}

#[derive(Serialize)]
pub struct GetHolderEntryInfoResponse {
    #[serde(with = "string_as_number")]
    pub amount_sol_lamp: u64,
    pub token: TokenEntryInfo,
    pub rank: i64,
}

#[derive(Deserialize, Serialize)]
pub struct TokenEntryInfo {
    #[serde(with = "string_as_number")]
    pub total_dec: u64,  
    #[serde(with = "string_as_number")]
    pub vested_dec: u64,
    #[serde(with = "string_as_number")]
    pub claimed_dec: u64,
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
    pub premarket_account: String,// base58
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Network {
    #[serde(rename = "devnet")]
    Devnet,
    #[serde(rename = "mainnet-beta")]
    MainnetBeta,
}
impl fmt::Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Network::Devnet => "devnet",
            Network::MainnetBeta => "mainnet-beta",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommonTxFields {
    pub network: Network, // todo: remove me and set from env
    pub unsigned_tx: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "tx_type", rename_all = "snake_case")]
pub enum TxToSignRequest {
    CreatePremarket(CommonTxFields),
    JoinPremarket(CommonTxFields),
    OutOfPremarket(CommonTxFields),

    // todo: OldTxFields - fix me to CommonTxFields
    FinishPremarket(OldTxFields),
    ExtendPremarket(OldTxFields),
    UpdateUri(OldTxFields),
    ClaimTokens(OldTxFields),
    RefundPremarket(OldTxFields),

    WithdrawVesting(CommonTxFields),
}

#[derive(Debug, Clone, Deserialize)]
pub struct OldTxFields {
    #[serde(flatten)]
    pub common: CommonTxFields,
    pub premarket: String,
}
impl TxToSignRequest {
    pub fn common(&self) -> &CommonTxFields {
        match self {
            TxToSignRequest::CreatePremarket(x) => x,
            TxToSignRequest::JoinPremarket(x) => x,
            TxToSignRequest::OutOfPremarket(x) => x,
            TxToSignRequest::FinishPremarket(x) => &x.common,
            TxToSignRequest::ExtendPremarket(x) => &x.common,
            TxToSignRequest::UpdateUri(x) => &x.common,
            TxToSignRequest::ClaimTokens(x) => &x.common,
            TxToSignRequest::RefundPremarket(x) => &x.common,
            TxToSignRequest::WithdrawVesting(x) => x,
        }
    }

    pub fn tx_type_str(&self) -> &'static str {
        match self {
            TxToSignRequest::CreatePremarket(_) => "create_premarket",
            TxToSignRequest::JoinPremarket(_) => "join_premarket",
            TxToSignRequest::OutOfPremarket(_) => "out_of_premarket",
            TxToSignRequest::FinishPremarket(_) => "finish_premarket",
            TxToSignRequest::ExtendPremarket(_) => "extend_premarket",
            TxToSignRequest::UpdateUri(_) => "update_uri",
            TxToSignRequest::ClaimTokens(_) => "claim_tokens",
            TxToSignRequest::RefundPremarket(_) => "refund_premarket",
            TxToSignRequest::WithdrawVesting(_) => "withdraw_vesting",
        }
    }
}


#[derive(serde::Serialize)]
pub struct SentTxResponse  {
    pub signature: String,
    pub status: TransactionStatus,
}

#[derive(Deserialize)]
pub struct CreatePremarketConceptTokenInfo{
    pub name: String,
    pub description: String,
    pub symbol: String,
    pub image_url: String,
    pub links: TokenLinksDTO,
    pub deadline: i64,           // unix sec
    #[serde(with = "string_as_number")]
    pub goal_sol_lamp: u64,
    #[serde(with = "string_as_number")]
    pub max_sol_lamp: u64,
    #[serde(with = "string_as_number")]
    pub creator_allocate_lamp: u64,
}

#[derive(Deserialize)]
pub struct CreatePremarketConceptRequest {
    pub network: String,         // "devnet" | "mainnet-beta"
    pub user_pubkey: String,     // base58
    pub token_info: CreatePremarketConceptTokenInfo,
}

#[derive(Serialize)]
pub struct CreatePremarketConceptResponse {
    pub premarket_account_pda: String, // base58
    pub premarket_id: Uuid, 
}

#[derive(Deserialize)]
pub struct CreatePremarketTxRequest {
    pub network: String,         // "devnet" | "mainnet-beta"
    pub user_pubkey: String,     // base58
    pub uri: String,
    pub image_url: String,
    pub premarket_pubkey: String,
    #[serde(with = "string_as_number")]
    pub creator_allocate_lamp: u64,
}

#[derive(Serialize)]
pub struct CreatePremarketTxResponse {
    pub transaction: String,           // base64(serialized Transaction)
    pub premarket_account_pda: String, // base58
    pub mint_address: String,
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
