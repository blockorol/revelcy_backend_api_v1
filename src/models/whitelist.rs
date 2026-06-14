use anyhow::bail;

use crate::models::user::User;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistUserInfo {
    pub user: User,
    pub status: WhitelistStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistUsersResult {
    pub items: Vec<WhitelistUserInfo>,
    pub total: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WhitelistStatus {
    Requested,
    Approved,
    Rejected,
}

impl WhitelistStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            WhitelistStatus::Requested => "REQUESTED",
            WhitelistStatus::Approved => "APPROVED",
            WhitelistStatus::Rejected => "REJECTED",
        }
    }
}

impl FromStr for WhitelistStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "REQUESTED" => Ok(Self::Requested),
            "APPROVED" => Ok(Self::Approved),
            "REJECTED" => Ok(Self::Rejected),
            _ => bail!("invalid whitelist status: {}", s),
        }
    }
}
