use serde::{Deserialize, Serialize};

// ======= HTTP DTO Public =======

#[derive(Serialize)]
pub struct StartSessionResponseDto {
    pub nonce: String,
    pub jwt: String,
}

#[derive(Deserialize)]
pub struct ConfirmLoginRequestDto {
    pub wallet_address: String,
    pub signature: String,
    pub jwt: String,
}

#[derive(Serialize)]
pub struct ConfirmLoginResponseDto {
    pub jwt: String,
    pub is_new_user: bool,
}

#[derive(Serialize)]
pub struct WalletInfoResponseDto {
    pub creation_time: String,
    pub balance: f64,
    pub tx_amount: String,
}

#[derive(Serialize)]
pub struct PremarketInfoResponseDto {
    pub users: Vec<(String, u64, bool)>,
    pub end_timestamp: i64,
    pub goal_sol: u64,
    pub max_sol: u64,
    pub mint: String,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub creator: String,
}
