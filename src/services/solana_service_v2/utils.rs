use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use bs58;
use sha2::{Digest, Sha256};
use solana_client::nonblocking::rpc_client::RpcClient as AsyncRpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    hash::Hash,
    instruction::CompiledInstruction,
    message::Message,
    pubkey::Pubkey,
};

use serde_json;

fn pk(s: &str) -> Pubkey {
    Pubkey::from_str(s).expect("invalid pubkey")
}

#[inline]
fn assert_len_64(bytes: &[u8], label: &str) {
    if bytes.len() != 64 {
        panic!("expected 64 bytes for {}, got {}", label, bytes.len());
    }
}

pub fn anchor_sighash_global(name: &str) -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(b"global:");
    hasher.update(name.as_bytes());
    let hash = hasher.finalize();
    let mut out = [0u8; 8];
    out.copy_from_slice(&hash[..8]);
    out
}

pub async fn get_valid_latest_blockhash(client: &AsyncRpcClient, max_retries: usize) -> Result<Hash> {
    let mut retries = 0;
    let mut recent_blockhash = client.get_latest_blockhash().await.context("Failed to get blockhash")?;

    loop {
        match client.is_blockhash_valid(&recent_blockhash, CommitmentConfig::processed()).await {
            Ok(true) => return Ok(recent_blockhash),
            Ok(false) | Err(_) => {
                retries += 1;
                if retries >= max_retries {
                    return Err(anyhow!("Blockhash still invalid after {} retries", max_retries));
                }
                recent_blockhash = client.get_latest_blockhash().await.context("Failed to get blockhash")?;
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        }
    }
}

// ===== parsing helpers (used by parse_create_premarket_tx_from_base64)

pub fn find_anchor_instruction<'a>(
    msg: &'a Message,
    program_id: &Pubkey,
    expected_sighash: &[u8; 8],
) -> Result<&'a CompiledInstruction> {
    for ix in &msg.instructions {
        let pid = *msg
            .account_keys
            .get(ix.program_id_index as usize)
            .ok_or_else(|| anyhow!("program_id_index out of bounds"))?;

        if &pid != program_id {
            continue;
        }
        if ix.data.len() < 8 {
            continue;
        }
        let sighash: [u8; 8] = ix.data[0..8].try_into().unwrap();
        if &sighash == expected_sighash {
            return Ok(ix);
        }
    }
    Err(anyhow!("no matching anchor instruction"))
}

pub fn resolve_account(msg: &Message, account_index: usize) -> Result<Pubkey> {
    msg.account_keys
        .get(account_index)
        .copied()
        .ok_or_else(|| anyhow!("account index out of bounds"))
}

// ===== small helper for mint key parsing (used by build_create_premarket_tx_unsigned)
pub fn parse_privkey_64(s: &str) -> Result<Vec<u8>> {
    let s = s.trim();

    if s.starts_with('[') && s.ends_with(']') {
        let v: Vec<u8> = serde_json::from_str(s).context("invalid JSON priv_key")?;
        anyhow::ensure!(v.len() == 64, "json priv_key must be 64 bytes, got {}", v.len());
        return Ok(v);
    }

    if s.contains(',') && !s.contains(':') && !s.contains('[') && !s.contains(']') {
        let v: Result<Vec<u8>, _> = s.split(',').map(|x| x.trim().parse::<u8>()).collect();
        let v = v.context("invalid CSV priv_key (non-numeric token)")?;
        anyhow::ensure!(v.len() == 64, "csv priv_key must be 64 bytes, got {}", v.len());
        return Ok(v);
    }

    if let Some(b64) = s.strip_prefix("base64:") {
        let v = BASE64.decode(b64).context("invalid base64 priv_key")?;
        anyhow::ensure!(v.len() == 64, "base64 priv_key must be 64 bytes, got {}", v.len());
        return Ok(v);
    }

    if s.contains('=') || s.contains('/') || s.contains('+') {
        if let Ok(v) = BASE64.decode(s) {
            anyhow::ensure!(v.len() == 64, "base64 priv_key must be 64 bytes, got {}", v.len());
            return Ok(v);
        }
    }

    let v = bs58::decode(s).into_vec().context("invalid base58 priv_key")?;
    anyhow::ensure!(v.len() == 64, "base58 priv_key must be 64 bytes, got {}", v.len());
    Ok(v)
}
