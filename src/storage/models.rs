
use anyhow::{anyhow, Result, Context};
use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;
use std::convert::TryFrom;
use crate::models::user::User;
use crate::models::premarket::{ UserInfoShort, PremarketInfoServiceModel, PremarketGoal, TokenLinks, TokenInfo, PremarketState};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WhitelistDbModel {
    pub id: Uuid,
    pub premarket_id: Uuid,
    pub user_id: Uuid,
}


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


impl TryFrom<UserDbModel> for User {
    type Error = anyhow::Error;

    fn try_from(u: UserDbModel) -> Result<Self> {
        Ok(Self {
            id: u.id,
            username: u.username,
            avatar_url: u.avatar_url,
            wallets: vec![],
        })
    }
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
    pub is_whitelist_enabled: bool,
    pub premarket_created: i64,
    pub premarket_finished: Option<i64>,

    pub state: String, // лучше использовать enum, но можно и строку
}

impl TryFrom<PremarketInfoDbModel> for PremarketInfoServiceModel {
    type Error = anyhow::Error;

    fn try_from(pm_db: PremarketInfoDbModel) -> Result<Self> {
        // i64 → u64 (lamports is never negative)
        let solana_lamp = u64::try_from(pm_db.premarket_goal_sol_lamp)
            .context("premarket_goal_sol_lamp must be non-negative")?;

        // state: String → PremarketState
        let state = pm_db
            .state
            .parse::<PremarketState>()
            .map_err(|_| anyhow!("invalid premarket state: {}", pm_db.state))?;


        Ok(Self {
            id: pm_db.id,
            blockchain_address: pm_db.bc_address,
            short_url_name: pm_db.short_url_name,

            creator: UserInfoShort {
                id: pm_db.creator_id,
                blockchain_address: pm_db.creator_address,
            },

            token_info: TokenInfo {
                address: pm_db.mint_address,
                name: pm_db.name,
                description: pm_db.description,
                symbol: pm_db.symbol,
                image_url: pm_db.image_url,
                data_uri: pm_db.data_uri,
                links: TokenLinks {
                    telegram: pm_db.telegram,
                    twitter: pm_db.twitter,
                    web_site: pm_db.web_site,
                },
            },
            goal: PremarketGoal {
                solana_lamp: solana_lamp as i64,
            },

            deadline_timestamp: pm_db.premarket_deadline,
            created_timestamp: pm_db.premarket_created,
            finished_timestamp: pm_db.premarket_finished,

            is_extended: pm_db.is_extended,
            is_hided: pm_db.is_hided,
            is_whitelist_enabled: pm_db.is_whitelist_enabled,
            state,
        })
    }
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

}

pub struct HolderStats {
    pub holders: Vec<HolderDbModel>,
    pub total_active_count: i64,
    pub reserved_sol_lamp: i64,
    pub reserved_sol_24h_before_lamp: i64,
}

#[derive(sqlx::FromRow)]
pub struct BondingPostionDbModel {
    pub holder_amount: Option<i64>,
    pub total_amount: i64,
    pub is_claimed: bool,
}