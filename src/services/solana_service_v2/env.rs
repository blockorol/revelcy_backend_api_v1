use crate::models::premarket::SolanaNetwork;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair},
};
use std::path::Path;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use bs58;
use super::utils::{assert_len_64, pk};

pub fn read_revelcy_auth(network: SolanaNetwork) -> Keypair {
    let var = match network {
        SolanaNetwork::Devnet => "REVELCY_AUTH_PRIVATE_KEY_DEV",
        SolanaNetwork::MainnetBeta => "REVELCY_AUTH_PRIVATE_KEY_MAIN",
    };

    let raw = std::env::var(var).unwrap_or_else(|_| panic!("env {var} required"));
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
    match network {
        SolanaNetwork::Devnet => std::env::var("SOLANA_DEVNET_RPC")
            .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string()),
        SolanaNetwork::MainnetBeta => std::env::var("SOLANA_MAINNET_RPC")
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
    }
}

pub fn program_id_for(network: SolanaNetwork) -> Pubkey {
    let (env_key, fallback) = match network {
        SolanaNetwork::Devnet => ("PURPLE_PROGRAM_ID_DEV", "AUf85EmXsTYGGgQnYJR2heKCxkTf5WtnKFvpkLtQ58sG"),
        SolanaNetwork::MainnetBeta => ("PURPLE_PROGRAM_ID_MAIN", "AtuMMXXjyAW3fSJrnWYon1ynxUA7CyQ3Qz2E6JqiTmhu"),
    };

    match std::env::var(env_key) {
        Ok(v) => Pubkey::from_str(&v).unwrap_or_else(|_| panic!("invalid {} pubkey: {}", env_key, v)),
        Err(_) => {
            eprintln!("WARN: {} not set; using fallback {}", env_key, fallback);
            pk(fallback)
        }
    }
}
