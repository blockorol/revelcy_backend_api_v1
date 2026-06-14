use super::utils::{assert_len_64, pk};
use crate::config;
use crate::models::premarket::SolanaNetwork;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use bs58;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair},
};
use std::path::Path;
use std::str::FromStr;

pub fn read_revelcy_auth(network: SolanaNetwork) -> Keypair {
    let raw = match network {
        SolanaNetwork::Devnet => config::get_revelcy_auth_private_key_dev().unwrap_or_else(|_| {
            panic!("env {} required", config::REVELCY_AUTH_PRIVATE_KEY_DEV_ENV)
        }),
        SolanaNetwork::MainnetBeta => {
            config::get_revelcy_auth_private_key_main().unwrap_or_else(|_| {
                panic!("env {} required", config::REVELCY_AUTH_PRIVATE_KEY_MAIN_ENV)
            })
        }
    };
    let s = raw.trim();

    if s.ends_with(".json") || Path::new(s).exists() {
        return read_keypair_file(s).expect("failed to read keypair file");
    }

    if s.starts_with('[') {
        let bytes: Vec<u8> = serde_json::from_str(s).expect("invalid json keypair bytes");
        assert_len_64(&bytes, "json bytes");
        return Keypair::from_bytes(&bytes).expect("invalid keypair bytes (json)");
    }

    if let Some(b64) = s.strip_prefix("base64:") {
        let bytes = BASE64.decode(b64).expect("invalid base64 key");
        assert_len_64(&bytes, "base64 bytes");
        return Keypair::from_bytes(&bytes).expect("invalid keypair bytes (base64)");
    }
    if s.contains('=') || s.contains('/') || s.contains('+') {
        if let Ok(bytes) = BASE64.decode(s) {
            assert_len_64(&bytes, "base64 bytes");
            return Keypair::from_bytes(&bytes).expect("invalid keypair bytes (base64)");
        }
    }

    let bytes = bs58::decode(s).into_vec().expect("invalid base58 key");
    assert_len_64(&bytes, "base58 bytes");
    Keypair::from_bytes(&bytes).expect("invalid keypair bytes (base58)")
}

pub fn rpc_url(network: SolanaNetwork) -> String {
    crate::services::solana_rpc_client::rpc_url_for_network(network)
}

pub fn program_id_for(network: SolanaNetwork) -> Pubkey {
    let (env_key, configured, fallback) = match network {
        SolanaNetwork::Devnet => (
            config::PURPLE_PROGRAM_ID_DEV_ENV,
            config::get_purple_program_id_dev(),
            "AUf85EmXsTYGGgQnYJR2heKCxkTf5WtnKFvpkLtQ58sG",
        ),
        SolanaNetwork::MainnetBeta => (
            config::PURPLE_PROGRAM_ID_MAIN_ENV,
            config::get_purple_program_id_main(),
            "AtuMMXXjyAW3fSJrnWYon1ynxUA7CyQ3Qz2E6JqiTmhu",
        ),
    };

    match configured {
        Some(v) => {
            Pubkey::from_str(&v).unwrap_or_else(|_| panic!("invalid {} pubkey: {}", env_key, v))
        }
        None => {
            tracing::error!("WARN: {} not set; using fallback {}", env_key, fallback);
            pk(fallback)
        }
    }
}
