use chrono::{DateTime, Utc};
use solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_transaction_status::UiTransactionEncoding;
use std::str::FromStr;

use crate::constants::MAX_AMOUNT_SIG;

pub fn get_signatures_for_wallet(client: &RpcClient, wallet_pubkey: &Pubkey) -> Vec<String> {
    let mut all_signatures = Vec::new();
    let mut before = None;

    loop {
        if all_signatures.len() >= MAX_AMOUNT_SIG as usize {
            tracing::info!(
                "Reached {:?} signatures, stopping further fetch.",
                MAX_AMOUNT_SIG
            );
            break;
        }
        match client.get_signatures_for_address_with_config(
            &wallet_pubkey,
            GetConfirmedSignaturesForAddress2Config {
                before,
                limit: Some(1000),
                ..Default::default()
            },
        ) {
            Ok(signatures) => {
                if signatures.is_empty() {
                    break; // No more signatures to fetch
                }

                // Collect signatures and update the `before` parameter
                let sig_str = signatures.last().unwrap().signature.clone();
                before = Some(Signature::from_str(&sig_str).unwrap());
                all_signatures.extend(signatures.into_iter().map(|s| s.signature));
            }
            Err(err) => {
                tracing::error!("Failed to fetch signatures: {}", err);
                std::process::exit(1);
            }
        }
    }

    tracing::info!("Signatures for wallet {}:", wallet_pubkey);
    tracing::info!("Total signatures: {}", all_signatures.len());

    return all_signatures;
}

pub fn get_creation_time(
    client: &RpcClient,
    all_signatures: &Vec<String>,
) -> Option<DateTime<Utc>> {
    if let Some(last_signature) = all_signatures.last() {
        let signature = Signature::from_str(last_signature).unwrap();
        let config = RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::Json),
            commitment: None,
            max_supported_transaction_version: Some(0),
        };
        match client.get_transaction_with_config(&signature, config) {
            Ok(transaction) => {
                tracing::info!("Last signature: {}", last_signature);
                tracing::info!("Block number (slot): {}", transaction.slot);
                if let Some(block_time) = transaction.block_time {
                    // Convert Unix timestamp to DateTime<Utc>
                    let dt_first_sig =
                        DateTime::<Utc>::from_timestamp(block_time, 0).unwrap_or_default();
                    return Some(dt_first_sig);
                } else {
                    tracing::info!("Timestamp: Not available");
                    return None;
                }
            }
            Err(err) => {
                tracing::error!(
                    "Failed to fetch transaction details for the last signature: {}",
                    err
                );
                return None;
            }
        }
    } else {
        tracing::info!("No signatures found.");
        return None;
    }
}
