use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use bincode;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    message::Message,
    pubkey::Pubkey,
    signature::Signer,
    system_program,
    transaction::Transaction,
};

use crate::models::premarket::{BuildJoinTxParams, BuiltTx, SolanaNetwork};

use super::constants::JOIN_METHOD_NAME;
use super::env::{program_id_for, read_revelcy_auth};
use super::solana_methods::make_async_rpc_client;
use super::utils::{
    anchor_sighash_global, find_anchor_instruction, get_valid_latest_blockhash, resolve_account,
};

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
struct JoinArgsBorsh {
    amount_in_lamports: u64,
}

#[derive(Debug, Clone)]
pub struct ParsedJoinPremarketTx {
    pub revelcy_auth: Pubkey,
    pub user: Pubkey,
    pub premarket: Pubkey,
    pub params: BuildJoinTxParams,
}

// join → unsigned
pub async fn build_join_premarket_tx_unsigned(params: BuildJoinTxParams) -> Result<BuiltTx> {
    let program_id = program_id_for(params.network);
    let rpc = make_async_rpc_client(params.network);

    let mut data = Vec::with_capacity(8 + 16);
    data.extend_from_slice(&anchor_sighash_global(JOIN_METHOD_NAME));

    JoinArgsBorsh {
        amount_in_lamports: params.amount,
    }
    .serialize(&mut data)
    .map_err(|e| anyhow!("borsh serialize JoinArgs failed: {e}"))?;

    let revelcy = read_revelcy_auth(params.network);
    let revelcy_pub = revelcy.pubkey();

    // IDL:
    // 1) revelcy_auth (writable, signer)
    // 2) user         (writable, signer)
    // 3) premarket_account (writable)
    // 4) system_program
    let accounts = vec![
        AccountMeta::new(revelcy_pub, true),
        AccountMeta::new(params.user, true),
        AccountMeta::new(params.premarket, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];

    let ix = Instruction {
        program_id,
        accounts,
        data,
    };

    let blockhash = get_valid_latest_blockhash(&rpc, 50)
        .await
        .context("get_latest_blockhash failed")?;

    let msg = Message::new(&[ix], Some(&params.user));
    let mut tx = Transaction::new_unsigned(msg);
    tx.message.recent_blockhash = blockhash;

    let raw = bincode::serialize(&tx).context("bincode serialize(Transaction) failed")?;
    let tx_b64 = BASE64.encode(raw);

    Ok(BuiltTx {
        tx_base64: tx_b64,
        premarket_pda: Some(params.premarket),
    })
}

pub fn parse_join_premarket_tx_from_base64(
    tx_b64: &str,
    network: SolanaNetwork,
) -> Result<ParsedJoinPremarketTx> {
    let program_id = program_id_for(network);
    let expected_sighash = anchor_sighash_global(JOIN_METHOD_NAME);

    let raw = BASE64.decode(tx_b64).context("tx_base64 decode failed")?;
    let tx: Transaction =
        bincode::deserialize(&raw).context("bincode deserialize(Transaction) failed")?;
    let msg: &Message = &tx.message;

    let ix = find_anchor_instruction(msg, &program_id, &expected_sighash)
        .context("join_to_premarket instruction not found")?;

    if ix.data.len() < 8 {
        return Err(anyhow!("instruction data too short (<8)"));
    }

    let args =
        JoinArgsBorsh::try_from_slice(&ix.data[8..]).context("borsh decode JoinArgs failed")?;

    // accounts: revelcy_auth, user, premarket, system_program
    if ix.accounts.len() < 4 {
        return Err(anyhow!("instruction accounts too short (<4)"));
    }

    let revelcy_auth = resolve_account(msg, ix.accounts[0] as usize)?;
    let user = resolve_account(msg, ix.accounts[1] as usize)?;
    let premarket = resolve_account(msg, ix.accounts[2] as usize)?;
    let system_program_key = resolve_account(msg, ix.accounts[3] as usize)?;

    if system_program_key != system_program::ID {
        return Err(anyhow!("invalid system_program account"));
    }

    let params = BuildJoinTxParams {
        network,
        user,
        premarket,
        amount: args.amount_in_lamports,
    };

    Ok(ParsedJoinPremarketTx {
        revelcy_auth,
        user,
        premarket,
        params,
    })
}
