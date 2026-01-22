use serde::{Serialize, Deserialize};
use crate::models::user::User;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistUsersResult {
    pub items: Vec<User>,
    pub total: Option<i64>,
}
