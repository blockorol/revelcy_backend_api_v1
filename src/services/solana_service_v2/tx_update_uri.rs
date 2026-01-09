use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use bincode;
use borsh::BorshDeserialize;
use solana_sdk::{message::Message, pubkey::Pubkey, system_program, transaction::Transaction};

use crate::models::premarket::{SolanaNetwork, BuiltTx};

use super::constants::UPDATE_PREMARKET_DATA_METHOD_NAME;
use super::env::program_id_for;

use super::tx_update_premarket_data::{UpdatePremarketDataArgs, build_update_premarket_data_tx_unsigned};
use super::utils::{anchor_sighash_global, find_anchor_instruction, resolve_account};

#[derive(Debug, Clone)]
pub struct ParsedUpdateURIPremarketTx {
    pub revelcy_auth: Pubkey,
    pub user: Pubkey,
    pub premarket: Pubkey,
    pub new_uri: String,
}

pub fn parse_update_uri_premarket_tx_from_base64(
    tx_b64: &str,
    network: SolanaNetwork,
) -> Result<ParsedUpdateURIPremarketTx> {
    let program_id = program_id_for(network);
    let expected_sighash = anchor_sighash_global(UPDATE_PREMARKET_DATA_METHOD_NAME);

    // 1) base64 → bytes
    let raw = BASE64.decode(tx_b64).context("tx_base64 decode failed")?;

    // 2) deserialize tx
    let tx: Transaction =
        bincode::deserialize(&raw).context("bincode deserialize(Transaction) failed")?;
    let msg: &Message = &tx.message;

    // 3) find anchor ix (update_premarket_data)
    let ix = find_anchor_instruction(msg, &program_id, &expected_sighash)
        .context("update_premarket_data instruction not found")?;

    // 4) decode borsh args (skip 8 bytes discriminator)
    if ix.data.len() < 8 {
        return Err(anyhow!("instruction data too short (<8)"));
    }

    let args = UpdatePremarketDataArgs::try_from_slice(&ix.data[8..])
        .context("borsh decode UpdatePremarketDataArgs failed")?;

    // 5) resolve accounts
    // expected accounts (as in build_update_premarket_data_tx_unsigned):
    // 0) revelcy_auth (signer)
    // 1) user (signer)
    // 2) premarket (writable)
    // 3) system_program
    if ix.accounts.len() < 4 {
        return Err(anyhow!("instruction accounts too short (<4)"));
    }

    let revelcy_auth = resolve_account(msg, ix.accounts[0] as usize)?;
    let user = resolve_account(msg, ix.accounts[1] as usize)?;
    let premarket = resolve_account(msg, ix.accounts[2] as usize)?;
    let sys = resolve_account(msg, ix.accounts[3] as usize)?;

    if sys != system_program::ID {
        return Err(anyhow!(
            "invalid system_program account (expected system_program::ID)"
        ));
    }

    // 6) validate "extend" shape inside update args
    let new_uri = args
        .uri
        .ok_or_else(|| anyhow!("update uri tx: new uri must be Some(..)"))?;

    // optional strictness (рекомендую, чтобы нельзя было подсунуть update с лишними полями)
    if args.goal_sol.is_some()
        || args.max_sol.is_some()
        || args.mint.is_some()
        || args.name.is_some()
        || args.symbol.is_some()
        || args.end_timestamp.is_some()
        || args.end_timestamp_updated.is_some()
        || args.creator.is_some()
    {
        return Err(anyhow!("extend tx: only end_timestamp fields must be set"));
    }

    Ok(ParsedUpdateURIPremarketTx {
        revelcy_auth,
        user,
        premarket,
        new_uri,
    })
}

pub async fn build_update_uri_premarket_tx_unsigned(
    network: SolanaNetwork,
    user: Pubkey,
    premarket: Pubkey,
    new_uri: String,
) -> Result<BuiltTx> {
    let args = UpdatePremarketDataArgs {
        end_timestamp: None,
        end_timestamp_updated: None,
        goal_sol: None,
        max_sol: None,
        mint: None,
        name: None,
        symbol: None,
        uri: Some(new_uri),
        creator: None,
    };
    build_update_premarket_data_tx_unsigned(network, user, premarket, args).await
}
