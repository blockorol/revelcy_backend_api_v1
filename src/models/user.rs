use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub wallets: Vec<String>,
}

pub struct Session {
    pub nonce: String,
}

pub struct WalletAddress {
    pub address: String,
}
