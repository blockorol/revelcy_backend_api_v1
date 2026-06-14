use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use bincode;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    message::Message,
    pubkey::Pubkey,
    signature::Signer,
    system_program,
    transaction::Transaction,
};

use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::ID as associated_token_program_id;
use spl_token::ID as token_program_id;

use crate::services::solana_service_v2::vesting::generate_vesting_pda;

use crate::models::premarket::{BuildWithdrawVestingTxParams, BuiltTx, SolanaNetwork};

use super::constants::WITHDRAW_VESTING_METHOD_NAME;
use super::env::{program_id_for, read_revelcy_auth};
use super::solana_methods::make_async_rpc_client;
use super::utils::{
    anchor_sighash_global, find_anchor_instruction, get_valid_latest_blockhash, resolve_account,
};

#[derive(Debug, Clone)]
pub struct ParsedWithdrawVestingTx {
    pub user: Pubkey,
    pub revelcy_auth: Pubkey,
    pub token_mint: Pubkey,
    pub vesting_account: Pubkey,
}

pub async fn build_withdraw_vesting_tx_unsigned(
    params: BuildWithdrawVestingTxParams,
) -> Result<BuiltTx> {
    let program_id = program_id_for(params.network);
    let rpc = make_async_rpc_client(params.network);
    let revelcy_auth = read_revelcy_auth(params.network);

    // Calculate vesting_account PDA: seeds = ["vesting", token_mint]
    let vesting_account = generate_vesting_pda(params.network, params.token_mint);
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
        premarket_pda: None,
    })
}

pub fn parse_withdraw_vesting_tx_from_base64(
    tx_b64: &str,
    network: SolanaNetwork,
) -> Result<ParsedWithdrawVestingTx> {
    let program_id = program_id_for(network);
    let expected_sighash = anchor_sighash_global(WITHDRAW_VESTING_METHOD_NAME);

    // 1) base64 → bytes
    let raw = BASE64.decode(tx_b64).context("tx_base64 decode failed")?;

    // 2) deserialize tx
    let tx: Transaction =
        bincode::deserialize(&raw).context("bincode deserialize(Transaction) failed")?;
    let msg: &Message = &tx.message;

    // 3) find anchor ix (withdraw_vesting)
    let ix = find_anchor_instruction(msg, &program_id, &expected_sighash)
        .context("withdraw_vesting instruction not found")?;

    // 4) validate args (WithdrawVestingArgs is empty, so only discriminator)
    if ix.data.len() != 8 {
        return Err(anyhow!(
            "instruction data should be exactly 8 bytes (discriminator only)"
        ));
    }

    // 5) resolve accounts
    // Expected accounts according to IDL:
    // 0) user (writable, signer)
    // 1) revelcy_auth (writable, signer)
    // 2) token_mint (readonly)
    // 3) vesting_account (writable, PDA)
    // 4) vesting_ata (writable)
    // 5) user_ata (writable)
    // 6) system_program (readonly)
    // 7) token_program (readonly)
    // 8) associated_token_program (readonly)
    if ix.accounts.len() < 9 {
        return Err(anyhow!("instruction accounts too short (<9)"));
    }

    let user = resolve_account(msg, ix.accounts[0] as usize)?;
    let revelcy_auth = resolve_account(msg, ix.accounts[1] as usize)?;
    let token_mint = resolve_account(msg, ix.accounts[2] as usize)?;
    let vesting_account = resolve_account(msg, ix.accounts[3] as usize)?;

    // Optional: validate system_program, token_program, associated_token_program
    let sys = resolve_account(msg, ix.accounts[6] as usize)?;
    if sys != system_program::ID {
        return Err(anyhow!(
            "invalid system_program account (expected system_program::ID)"
        ));
    }

    let token_prog = resolve_account(msg, ix.accounts[7] as usize)?;
    if token_prog != token_program_id {
        return Err(anyhow!("invalid token_program account"));
    }

    let assoc_token_prog = resolve_account(msg, ix.accounts[8] as usize)?;
    if assoc_token_prog != associated_token_program_id {
        return Err(anyhow!("invalid associated_token_program account"));
    }

    // Validate vesting_account PDA
    let expected_vesting_account = generate_vesting_pda(network, token_mint);

    if vesting_account != expected_vesting_account {
        return Err(anyhow!(
            "vesting_account mismatch: expected {}, got {}",
            expected_vesting_account,
            vesting_account
        ));
    }

    Ok(ParsedWithdrawVestingTx {
        user,
        revelcy_auth,
        token_mint,
        vesting_account,
    })
}
