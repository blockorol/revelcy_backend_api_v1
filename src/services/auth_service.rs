use crate::models::user::{User, Session, WalletAddress};
use actix_web::web;
use sqlx::PgPool;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use base64::{engine::general_purpose, Engine as _};
use std::str::FromStr;
use crate::storage::user_repo;
use chrono::Utc;


pub fn generate_session_rand() -> Session {
    use rand::{distributions::Alphanumeric, Rng};
    let nonce: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    Session { nonce }
}

pub fn generate_session() -> Session {
    let timestamp = Utc::now().timestamp_millis();
    let nonce = format!("Login to Revelcy at {}", timestamp);
    Session { nonce }
}

pub fn verify_signature(wallet: &WalletAddress, nonce: &str, signature_base64: &str) -> bool {
    match Pubkey::from_str(&wallet.address) {
        Ok(pubkey) => match general_purpose::STANDARD.decode(signature_base64) {
            Ok(signature_bytes) => {
                // Convert Vec<u8> to [u8; 64] and create Signature
                if signature_bytes.len() == 64 {
                    let sig_array: [u8; 64] = match signature_bytes.try_into() {
                        Ok(arr) => arr,
                        Err(_) => {
                            println!("Failed to convert signature bytes for wallet {}", wallet.address);
                            return false;
                        }
                    };
                    let signature = Signature::from(sig_array);
                    
                    if signature.verify(pubkey.as_ref(), nonce.as_bytes()) {
                        true
                    } else {
                        println!("Signature verification failed for wallet {}", wallet.address);
                        false
                    }
                } else {
                    println!("Invalid signature length: {} (expected 64) for wallet {}", 
                             signature_bytes.len(), wallet.address);
                    false
                }
            }
            Err(err) => {
                println!("Failed to decode signature: {} for wallet {}", err, wallet.address);
                false
            }
        },
        Err(err) => {
            println!("Invalid wallet address '{}': {}", wallet.address, err);
            false
        }
    }
}

pub async fn get_or_create_user_info(
    pool: web::Data<PgPool>,
    wallet_address: &str,
) -> Result<(User, bool), String> {
    match user_repo::get_user_by_wallet(&pool, wallet_address).await {
        Ok(Some(user)) => Ok((user, false)),
        Ok(None) => {
            match user_repo::create_user_with_wallet(&pool, wallet_address).await {
                Ok(user) => Ok((user, true)),
                Err(_) => Err("Failed to create user".to_string()),
            }
        }
        Err(_) => Err("Failed to fetch user".to_string()),
    }
}
