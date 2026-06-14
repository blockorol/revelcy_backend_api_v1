use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use bincode;
use solana_client::nonblocking::rpc_client::RpcClient as AsyncRpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    message::Message,
    signature::Signer,
    system_program,
    transaction::Transaction,
};
use std::time::Duration;

use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::ID as associated_token_program_id;
use spl_token::ID as token_program_id;

use crate::models::premarket::{BuildClaimTokensTxParams, BuiltTx};

use super::constants::CLAIM_TOKENS_METHOD_NAME;
use super::env::{program_id_for, read_revelcy_auth, rpc_url};
use super::utils::{anchor_sighash_global, get_valid_latest_blockhash};

pub async fn build_claim_tokens_tx_unsigned(params: BuildClaimTokensTxParams) -> Result<BuiltTx> {
    let program_id = program_id_for(params.network);
    let rpc = AsyncRpcClient::new_with_timeout(rpc_url(params.network), Duration::from_secs(15));
    let revelcy_auth = read_revelcy_auth(params.network);

    let revelcy_auth_ata = get_associated_token_address(&revelcy_auth.pubkey(), &params.token_mint);
    let user_ata = get_associated_token_address(&params.user, &params.token_mint);

    let accounts = vec![
        AccountMeta::new(revelcy_auth.pubkey(), true),
        AccountMeta::new(revelcy_auth_ata, false),
        AccountMeta::new(params.user, true),
        AccountMeta::new(user_ata, false),
        AccountMeta::new(params.premarket, false),
        AccountMeta::new_readonly(params.token_mint, false),
        AccountMeta::new_readonly(system_program::ID, false),
        AccountMeta::new_readonly(token_program_id, false),
        AccountMeta::new_readonly(associated_token_program_id, false),
    ];

    let mut data = Vec::with_capacity(8);
    data.extend_from_slice(&anchor_sighash_global(CLAIM_TOKENS_METHOD_NAME));

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

    let raw = bincode::serialize(&tx).context("serialize tx failed")?;
    let tx_b64 = BASE64.encode(raw);

    Ok(BuiltTx {
        tx_base64: tx_b64,
        premarket_pda: Some(params.premarket),
    })
}
