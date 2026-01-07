use serde::{Serialize, Deserialize};
use solana_sdk::pubkey::Pubkey;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub wallets: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UserContextData {
    pub internal_id: Uuid,
    pub current_pubkey: Pubkey,
    pub wallets: Vec<Pubkey>,
}

pub struct Session {
    pub nonce: String,
}

pub struct WalletAddress {
    pub address: String,
}
