use serde::{Serialize, Deserialize};
use crate::models::user::User;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistUsersResult {
    pub items: Vec<User>,
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

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "REQUESTED" => Ok(Self::Requested),
            "APPROVED" => Ok(Self::Approved),
            "REJECTED" => Ok(Self::Rejected),
            _ => bail!("invalid whitelist status: {}", s),
        }
    }
}
