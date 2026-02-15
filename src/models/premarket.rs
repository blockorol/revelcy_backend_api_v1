use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::str::FromStr;
use solana_sdk::pubkey::Pubkey;

use crate::api::premarket::{ TokenDynamicInfoDTO, HolderInfoDTO};
use crate::models::vesting::VestingSettingsServiceModel;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PremarketLookupKeyType {
    BcAddress,
    Name,
    Id
}



#[derive(Debug, Clone)]
pub struct BuildJoinTxParams {
    pub network: SolanaNetwork,
    pub user: solana_sdk::pubkey::Pubkey,
    pub premarket: solana_sdk::pubkey::Pubkey,
    pub amount: u64,
}

#[derive(Debug, Clone)]
pub struct BuildOutTxParams {
    pub network: SolanaNetwork,
    pub user: solana_sdk::pubkey::Pubkey,
    pub premarket: solana_sdk::pubkey::Pubkey,
}


#[derive(Debug, Clone)]
pub struct BuildFinishTxParams {
    pub network: SolanaNetwork,
    pub user: solana_sdk::pubkey::Pubkey,
    pub premarket: solana_sdk::pubkey::Pubkey,
    pub timestamp_start: i64,
    pub timestamp_end: i64,
    pub init_unlock: u64,
}

#[derive(Debug, Clone)]
pub struct BuildKillTxParams {
    pub network: SolanaNetwork,
    pub user: solana_sdk::pubkey::Pubkey,
    pub premarket: solana_sdk::pubkey::Pubkey,
    pub users: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BuildClaimTokensTxParams {
    pub network: SolanaNetwork,
    pub user: solana_sdk::pubkey::Pubkey,
    pub premarket: solana_sdk::pubkey::Pubkey,
    pub token_mint: solana_sdk::pubkey::Pubkey,
}

#[derive(Debug, Clone)]
pub struct BuildWithdrawVestingTxParams {
    pub network: SolanaNetwork,
    pub user: solana_sdk::pubkey::Pubkey,
    pub token_mint: solana_sdk::pubkey::Pubkey,
}

#[derive(Debug, Clone)]
pub struct GetPremarketDataParams {
    pub network: SolanaNetwork,
    pub premarket: solana_sdk::pubkey::Pubkey,
}


#[derive(Debug, Clone)]
pub struct BuildPremarketTxParams {
    pub premarket_pda: Pubkey,
    pub mint: Pubkey,
    pub network: SolanaNetwork,
    pub user: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub deadline: i64,
    pub goal: u64,
    pub max: u64,
    pub creator_allocate: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BuiltTx {
    pub tx_base64: String,
    pub premarket_pda: Option<Pubkey>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BuiltTxCreation {
    pub tx_base64: String,
    pub premarket_pda: Pubkey,
    pub mint_address: String
}

#[derive(Clone, Copy, Debug)]
pub enum SolanaNetwork {
    Devnet,
    MainnetBeta,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreatePremarketConceptModel {
    pub short_url_name: Option<String>,
    pub creator: UserInfoShort,
    pub token_info: TokenInfo,
    pub goal: PremarketGoal,
    pub deadline_timestamp: i64,
    pub created_timestamp: i64,
    pub is_hided: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreatePremarketInfoServiceModel {
    pub id: Option<Uuid>, // Option to create method
    pub blockchain_address: String,
    pub short_url_name: Option<String>,
    pub creator: UserInfoShort,
    pub token_info: TokenInfo,
    pub goal: PremarketGoal,
    pub deadline_timestamp: i64,
    pub created_timestamp: i64,
    pub finished_timestamp: Option<i64>,
    pub is_extended: bool,
    pub is_hided: bool,
    pub is_whitelist_enabled: bool,
    pub state: PremarketState,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PremarketInfoServiceModel {
    pub id: Uuid,
    pub blockchain_address: String,
    pub short_url_name: Option<String>,
    pub creator: UserInfoShort,
    pub token_info: TokenInfo,
    pub goal: PremarketGoal,
    pub deadline_timestamp: i64,
    pub created_timestamp: i64,
    pub finished_timestamp: Option<i64>,
    pub is_extended: bool,
    pub is_hided: bool,
    pub is_whitelist_enabled: bool,
    pub state: PremarketState,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserInfoShort {
    pub id: Uuid,
    pub blockchain_address: String,
}


#[derive(Serialize, Deserialize, Copy, Debug, Clone,PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PremarketState {
    Concept,
    Premarket,
    Canceled,
    Finished,
}
impl ToString for PremarketState {
    fn to_string(&self) -> String {
        match self {
            PremarketState::Concept => "concept",
            PremarketState::Premarket => "premarket",
            PremarketState::Canceled => "canceled",
            PremarketState::Finished => "finished",
        }
        .to_string()
    }
}
impl From<String> for PremarketState {
    fn from(s: String) -> Self {
        PremarketState::from_str(&s).unwrap_or(PremarketState::Premarket)
    }
}
impl FromStr for PremarketState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "concept" => Ok(PremarketState::Concept),
            "premarket" => Ok(PremarketState::Premarket),
            "canceled" => Ok(PremarketState::Canceled),
            "finished" => Ok(PremarketState::Finished),
            _ => Err(()),
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PremarketGoal {
    pub solana_lamp: i64,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenInfo {
    pub address: String,
    pub name: String,
    pub description: String,
    pub symbol: String,
    pub image_url: Option<String>,
    pub data_uri: String,
    pub links: TokenLinks,
}



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenLinks {
    pub telegram: Option<String>,
    pub twitter: Option<String>, 
    pub web_site: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CommunityInfoServiceModel {
    pub description: String,
    pub token_banner_url: Option<String>,
    pub links: Option<Vec<CommunityLink>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CommunityLink {
    pub text: String,
    pub url: String,
    pub r#type: LinkType,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum LinkType {
    X,
    Tg,
    Other,
}

#[derive(Serialize, Deserialize)]
pub struct FullPremarketInfo {
    pub main_info: PremarketInfoServiceModel,
    pub community: CommunityInfoServiceModel,
    pub vesting_settings: Option<VestingSettingsServiceModel>,
}


pub struct PremarketListResult {
    pub items: Vec<PremarketInfoServiceModel>,
    pub total: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenDynamicInfo {
    pub holders_count: u32,
    pub current_price_lamp: f64,
    pub reserved_sol_lamp: u64,
    pub change_24h: f64,
    pub holders: Vec<HolderInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HolderEntryInfo {
    pub amount_sol_lamp: u64,
    pub token: TokenEntryInfo,
    pub rank: i64,
}

#[derive(Serialize, Deserialize, Copy, Debug, Clone)]
pub struct BondingPostion {
    pub amount_sol_lamp: u64,
    pub before_amount_sol_lamp: u64,
    pub is_claimed: bool,
    pub rank: i64,
}

#[derive(Serialize, Deserialize, Copy, Debug, Clone)]
pub struct TokenEntryInfo {
    pub total_dec: u64,
    pub claimed_dec: u64,
    pub vested_dec: u64,

}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TxConfirmationStatusDTO {
    Pending,
    Confirmed,
    Failed,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HolderInfo {
    pub id: Option<Uuid>,
    pub wallet_address: String,
    pub join_timestamp: i64,
    pub icon_url: Option<String>,
    pub username: Option<String>,
    pub amount_sol_lamp: u64,
    pub claimed: bool,
}
impl From<TokenDynamicInfo> for TokenDynamicInfoDTO {
    fn from(info: TokenDynamicInfo) -> Self {
        TokenDynamicInfoDTO {
            holders_count: info.holders_count,
            current_price_lamp: info.current_price_lamp,
            reserved_sol_lamp: info.reserved_sol_lamp,
            change_24h: info.change_24h,
            holders: info.holders.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<HolderInfo> for HolderInfoDTO {
    fn from(holder: HolderInfo) -> Self {
        HolderInfoDTO {
            id: holder.id,
            wallet_address: holder.wallet_address,
            join_timestamp: holder.join_timestamp,
            icon_url: holder.icon_url,
            username: holder.username,
            amount_sol_lamp: holder.amount_sol_lamp,
            claimed: holder.claimed,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PremarketOnchainUser {
    pub wallet: Pubkey,
    pub contributed_lamports: u64,
    pub claimed: bool,
}

#[derive(Debug, Clone)]
pub struct PremarketOnchainData {
    pub users: Vec<PremarketOnchainUser>,
    pub end_timestamp: i64,
    pub extended_premarket: bool,
    pub goal_lamports: u64,
    pub max_lamports: u64,
    pub mint: Pubkey,
}

#[derive(Debug, Deserialize)]
pub struct PythPriceData {
    pub price: String,
    pub conf: String,
    pub expo: i32,
    pub publish_time: u64,
}

#[derive(Debug, Deserialize)]
pub struct PythEmaPriceData {
    pub price: String,
    pub conf: String,
    pub expo: i32,
    pub publish_time: u64,
}

#[derive(Debug, Deserialize)]
pub struct PythMetadata {
    pub slot: u64,
    pub proof_available_time: u64,
    pub prev_publish_time: u64,
}

#[derive(Debug, Deserialize)]
pub struct PythParsedData {
    pub id: String,
    pub price: PythPriceData,
    pub ema_price: PythEmaPriceData,
    pub metadata: PythMetadata,
}

#[derive(Debug, Deserialize)]
pub struct PythBinaryData {
    pub encoding: String,
    pub data: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PythResponse {
    pub binary: PythBinaryData,
    pub parsed: Vec<PythParsedData>,
}
