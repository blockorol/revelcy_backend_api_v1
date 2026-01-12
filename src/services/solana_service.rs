use anyhow::{Context, anyhow, Result};
use std::str::FromStr;
use solana_client::nonblocking::rpc_client::RpcClient as AsyncRpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    transaction::Transaction
};
use std::{time::Duration, path::Path};
use solana_transaction_status::UiTransactionEncoding;

use crate::models::premarket::{
    GetPremarketDataParams, 
    PremarketOnchainUser,
    PremarketOnchainData,
    SolanaNetwork, 
};

impl TryFrom<&str> for SolanaNetwork {
    type Error = anyhow::Error;
    fn try_from(s: &str) -> Result<Self> {
        match s {
            "devnet" => Ok(Self::Devnet),
            "mainnet-beta" | "mainnet_beta" | "mainnetbeta" => Ok(Self::MainnetBeta),
            _ => Err(anyhow!("unsupported network: {}", s)),
        }
    }
}

fn rpc_url(network: SolanaNetwork) -> String {
    match network {
        SolanaNetwork::Devnet =>
            std::env::var("SOLANA_DEVNET_RPC").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string()),
        SolanaNetwork::MainnetBeta =>
            std::env::var("SOLANA_MAINNET_RPC").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
    }
}

#[inline]
fn assert_len_64(bytes: &[u8], label: &str) {
    if bytes.len() != 64 {
        panic!("expected 64 bytes for {}, got {}", label, bytes.len());
    }
}


pub async fn get_premarket_data(
    params: GetPremarketDataParams,
) -> Result<PremarketOnchainData, Box<dyn std::error::Error>> {

    let client = AsyncRpcClient::new_with_timeout(rpc_url(params.network), Duration::from_secs(15));

    let account = client.get_account(&params.premarket).await?;
    
    println!("Account owner: {}", account.owner);
    println!("Account data length: {} bytes", account.data.len());
    
    // Skip the 8-byte discriminator
    let data = &account.data[8..];

    // Get the user vector length
    let users_len = u32::from_le_bytes(data[0..4].try_into()?);
    println!("\nUsers vector length: {}", users_len);
    
    // Calculate offset after the user vector
    // Format: 4 bytes for length + (users_len * (32 bytes for pubkey + 8 bytes for lamports + 1 byte for claimed))
    let offset_after_users = 4 + (users_len as usize * 41);
    let mut all_entered_users = Vec::<PremarketOnchainUser>::new();

    // Extract user data if available
    if users_len > 0 {
        for i in 0..users_len as usize {
            let user_offset = 4 + (i * 41); // 4 bytes for vector length + i * entry size
            let pubkey_bytes = &data[user_offset..user_offset+32];
            let pubkey = Pubkey::new_from_array(pubkey_bytes.try_into()?);

            let lamports_bytes = &data[user_offset+32..user_offset+40];
            let lamports = u64::from_le_bytes(lamports_bytes.try_into()?);

            let claimed = data[user_offset+40] != 0;

            let user = PremarketOnchainUser {
                wallet: pubkey,
                contributed_lamports: lamports,
                claimed,
            };
            all_entered_users.push(user);

            println!("User {}: {} contributed {} lamports, claimed: {}", i, pubkey, lamports, claimed);
        }
    }
    
    // Now extract the rest of the fields after the user vector
    let end_timestamp = i64::from_le_bytes(data[offset_after_users..offset_after_users+8].try_into()?);
    println!("End timestamp: {}", end_timestamp);
    
    let end_timestamp_updated = data[offset_after_users+8] != 0;
    println!("End timestamp updated: {}", end_timestamp_updated);
    
    let goal_sol = u64::from_le_bytes(data[offset_after_users+9..offset_after_users+17].try_into()?);
    println!("Goal SOL: {}", goal_sol);
    
    let max_sol = u64::from_le_bytes(data[offset_after_users+17..offset_after_users+25].try_into()?);
    println!("Max SOL: {}", max_sol);
    
    let mint = Pubkey::new_from_array(data[offset_after_users+25..offset_after_users+57].try_into()?);
    println!("Mint: {}", mint);

    Ok(PremarketOnchainData {
        users: all_entered_users,
        end_timestamp,
        extended_premarket: end_timestamp_updated,
        goal_lamports: goal_sol,
        max_lamports: max_sol,
        mint,
    })
}

