use crate::models::user::{Session, User, WalletAddress};
use crate::storage::user_repo;
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use sqlx::PgPool;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
pub enum AuthServiceError {
    FetchUser(sqlx::Error),
    CreateUser(sqlx::Error),
}

impl fmt::Display for AuthServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FetchUser(err) => write!(f, "failed to fetch user: {err}"),
            Self::CreateUser(err) => write!(f, "failed to create user: {err}"),
        }
    }
}

impl Error for AuthServiceError {}

pub type AuthServiceResult<T> = Result<T, AuthServiceError>;

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
                            tracing::info!(
                                "Failed to convert signature bytes for wallet {}",
                                wallet.address
                            );
                            return false;
                        }
                    };
                    let signature = Signature::from(sig_array);

                    if signature.verify(pubkey.as_ref(), nonce.as_bytes()) {
                        true
                    } else {
                        tracing::info!(
                            "Signature verification failed for wallet {}",
                            wallet.address
                        );
                        false
                    }
                } else {
                    tracing::info!(
                        "Invalid signature length: {} (expected 64) for wallet {}",
                        signature_bytes.len(),
                        wallet.address
                    );
                    false
                }
            }
            Err(err) => {
                tracing::info!(
                    "Failed to decode signature: {} for wallet {}",
                    err,
                    wallet.address
                );
                false
            }
        },
        Err(err) => {
            tracing::info!("Invalid wallet address '{}': {}", wallet.address, err);
            false
        }
    }
}

pub async fn get_or_create_user_info(
    pool: &PgPool,
    wallet_address: &str,
) -> AuthServiceResult<(User, bool)> {
    match user_repo::get_user_by_wallet(pool, wallet_address).await {
        Ok(Some(user)) => Ok((user, false)),
        Ok(None) => match user_repo::create_user_with_wallet(pool, wallet_address).await {
            Ok(user) => Ok((user, true)),
            Err(err) => Err(AuthServiceError::CreateUser(err)),
        },
        Err(err) => Err(AuthServiceError::FetchUser(err)),
    }
}
