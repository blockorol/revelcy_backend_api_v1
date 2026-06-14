use crate::config;
use crate::models::premarket::PublicPremarketInfo;
use crate::models::user::WalletInfo;
use crate::services::solana_rpc_client;
use solana_client::client_error::ClientError;
use solana_sdk::pubkey::Pubkey;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
pub enum PublicInfoServiceError {
    MissingSolanaRpc,
    InvalidWalletAddress,
    InvalidPremarketAddress,
    SolanaBalance(ClientError),
    SolanaAccount(ClientError),
    InvalidPremarketData(String),
}

impl PublicInfoServiceError {
    pub fn public_message(&self) -> String {
        match self {
            Self::MissingSolanaRpc => {
                format!("{} environment variable not set", config::SOLANA_RPC_ENV)
            }
            Self::InvalidWalletAddress => "Invalid wallet address format".to_string(),
            Self::InvalidPremarketAddress => "Invalid premarket account address format".to_string(),
            Self::SolanaBalance(err) => format!("Failed to get balance: {err}"),
            Self::SolanaAccount(err) => format!("Failed to get account: {err}"),
            Self::InvalidPremarketData(message) => message.clone(),
        }
    }
}

impl fmt::Display for PublicInfoServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.public_message())
    }
}

impl Error for PublicInfoServiceError {}

pub type PublicInfoServiceResult<T> = Result<T, PublicInfoServiceError>;

pub async fn get_wallet_info(pubkey: &str) -> PublicInfoServiceResult<WalletInfo> {
    let rpc = rpc_client()?;
    let pubkey =
        Pubkey::from_str(pubkey).map_err(|_| PublicInfoServiceError::InvalidWalletAddress)?;

    let balance_lamports = rpc
        .get_balance(&pubkey)
        .await
        .map_err(PublicInfoServiceError::SolanaBalance)?;

    let balance = format!("{:.2}", (balance_lamports as f64) / 1_000_000_000.0);
    let balance = balance.parse::<f64>().unwrap_or(0.0);

    Ok(WalletInfo {
        creation_time: "2025-01-01".to_string(),
        balance,
        tx_amount: "0".to_string(),
    })
}

pub async fn get_premarket_info(
    premarket_account: &str,
) -> PublicInfoServiceResult<PublicPremarketInfo> {
    let rpc = rpc_client()?;
    let premarket_account = Pubkey::from_str(premarket_account)
        .map_err(|_| PublicInfoServiceError::InvalidPremarketAddress)?;

    let account = rpc
        .get_account(&premarket_account)
        .await
        .map_err(PublicInfoServiceError::SolanaAccount)?;

    parse_premarket_info(&account.data)
}

fn rpc_client() -> PublicInfoServiceResult<solana_client::nonblocking::rpc_client::RpcClient> {
    solana_rpc_client::make_default_async_rpc_client()
        .map_err(|_| PublicInfoServiceError::MissingSolanaRpc)
}

fn parse_premarket_info(data: &[u8]) -> PublicInfoServiceResult<PublicPremarketInfo> {
    if data.len() < 8 {
        return Err(PublicInfoServiceError::InvalidPremarketData(
            "Premarket account data is too short".to_string(),
        ));
    }

    let data = &data[8..];
    let mut offset = 0usize;

    let users_len = read_u32(data, &mut offset)? as usize;
    let mut users = Vec::with_capacity(users_len);

    for _ in 0..users_len {
        let pubkey_bytes = read_bytes(data, &mut offset, 32)?;
        let pubkey = Pubkey::new_from_array(pubkey_bytes.try_into().map_err(|_| {
            PublicInfoServiceError::InvalidPremarketData("Invalid user pubkey bytes".to_string())
        })?)
        .to_string();

        let lamports =
            u64::from_le_bytes(read_bytes(data, &mut offset, 8)?.try_into().map_err(|_| {
                PublicInfoServiceError::InvalidPremarketData("Invalid lamports bytes".to_string())
            })?);

        let claimed = *read_bytes(data, &mut offset, 1)?.first().ok_or_else(|| {
            PublicInfoServiceError::InvalidPremarketData("Invalid claimed byte".to_string())
        })? != 0;

        users.push((pubkey, lamports, claimed));
    }

    let end_timestamp = read_i64(data, &mut offset)?;
    let goal_sol = read_u64(data, &mut offset)?;
    let max_sol = read_u64(data, &mut offset)?;

    let mint =
        Pubkey::new_from_array(read_bytes(data, &mut offset, 32)?.try_into().map_err(|_| {
            PublicInfoServiceError::InvalidPremarketData("Invalid mint bytes".to_string())
        })?)
        .to_string();

    let name = read_string(data, &mut offset, "name")?;
    let symbol = read_string(data, &mut offset, "symbol")?;
    let uri = read_string(data, &mut offset, "uri")?;

    let creator =
        Pubkey::new_from_array(read_bytes(data, &mut offset, 32)?.try_into().map_err(|_| {
            PublicInfoServiceError::InvalidPremarketData("Invalid creator bytes".to_string())
        })?)
        .to_string();

    Ok(PublicPremarketInfo {
        users,
        end_timestamp,
        goal_sol,
        max_sol,
        mint,
        name,
        symbol,
        uri,
        creator,
    })
}

fn read_u32(data: &[u8], offset: &mut usize) -> PublicInfoServiceResult<u32> {
    Ok(u32::from_le_bytes(
        read_bytes(data, offset, 4)?.try_into().map_err(|_| {
            PublicInfoServiceError::InvalidPremarketData("Invalid u32 bytes".to_string())
        })?,
    ))
}

fn read_u64(data: &[u8], offset: &mut usize) -> PublicInfoServiceResult<u64> {
    Ok(u64::from_le_bytes(
        read_bytes(data, offset, 8)?.try_into().map_err(|_| {
            PublicInfoServiceError::InvalidPremarketData("Invalid u64 bytes".to_string())
        })?,
    ))
}

fn read_i64(data: &[u8], offset: &mut usize) -> PublicInfoServiceResult<i64> {
    Ok(i64::from_le_bytes(
        read_bytes(data, offset, 8)?.try_into().map_err(|_| {
            PublicInfoServiceError::InvalidPremarketData("Invalid i64 bytes".to_string())
        })?,
    ))
}

fn read_string(data: &[u8], offset: &mut usize, field: &str) -> PublicInfoServiceResult<String> {
    let len = read_u32(data, offset)? as usize;
    let raw = read_bytes(data, offset, len)?;
    std::str::from_utf8(raw).map(str::to_string).map_err(|_| {
        PublicInfoServiceError::InvalidPremarketData(format!("Invalid UTF-8 in {field}"))
    })
}

fn read_bytes<'a>(
    data: &'a [u8],
    offset: &mut usize,
    len: usize,
) -> PublicInfoServiceResult<&'a [u8]> {
    let end = offset.checked_add(len).ok_or_else(|| {
        PublicInfoServiceError::InvalidPremarketData(
            "Premarket account data offset overflow".to_string(),
        )
    })?;
    if end > data.len() {
        return Err(PublicInfoServiceError::InvalidPremarketData(
            "Premarket account data is truncated".to_string(),
        ));
    }

    let value = &data[*offset..end];
    *offset = end;
    Ok(value)
}
