use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use bincode;
use solana_client::nonblocking::rpc_client::RpcClient as AsyncRpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    message::Message,
    pubkey::Pubkey,
    signature::Signer,
    system_program,
    transaction::Transaction,
};
use std::time::Duration;

use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::ID as associated_token_program_id;
use spl_token::ID as token_program_id;

use crate::models::premarket::{BuildWithdrawVestingTxParams, BuiltTx};

use super::constants::WITHDRAW_VESTING_METHOD_NAME;
use super::env::{program_id_for, read_revelcy_auth, rpc_url};
use super::utils::{anchor_sighash_global, get_valid_latest_blockhash};

pub async fn build_withdraw_vesting_tx_unsigned(params: BuildWithdrawVestingTxParams) -> Result<BuiltTx> {
    let program_id = program_id_for(params.network);
    let rpc = AsyncRpcClient::new_with_timeout(rpc_url(params.network), Duration::from_secs(15));
    let revelcy_auth = read_revelcy_auth(params.network);

    // Calculate vesting_account PDA: seeds = ["vesting", token_mint]
    let vesting_seed = b"vesting";
    let (vesting_account, _vesting_bump) = Pubkey::find_program_address(
        &[vesting_seed, params.token_mint.as_ref()],
        &program_id,
    );

    // Get associated token accounts
    let vesting_ata = get_associated_token_address(&vesting_account, &params.token_mint);
    let user_ata = get_associated_token_address(&params.user, &params.token_mint);

    // Build accounts according to IDL order:
    // 0. user (writable, signer)
    // 1. revelcy_auth (writable, signer)
    // 2. token_mint (readonly)
    // 3. vesting_account (writable, PDA)
    // 4. vesting_ata (writable)
    // 5. user_ata (writable)
    // 6. system_program (readonly)
    // 7. token_program (readonly)
    // 8. associated_token_program (readonly)
    let accounts = vec![
        AccountMeta::new(params.user, true),
        AccountMeta::new(revelcy_auth.pubkey(), true),
        AccountMeta::new_readonly(params.token_mint, false),
        AccountMeta::new(vesting_account, false),
        AccountMeta::new(vesting_ata, false),
        AccountMeta::new(user_ata, false),
        AccountMeta::new_readonly(system_program::ID, false),
        AccountMeta::new_readonly(token_program_id, false),
        AccountMeta::new_readonly(associated_token_program_id, false),
    ];

    // WithdrawVestingArgs is empty, so just the discriminator
    let mut data = Vec::with_capacity(8);
    data.extend_from_slice(&anchor_sighash_global(WITHDRAW_VESTING_METHOD_NAME));

    let ix = Instruction { program_id, accounts, data };

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
        premarket_pda: None,
    })
}

