use anyhow::Context;
use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use bincode;
use solana_client::nonblocking::rpc_client::RpcClient as AsyncRpcClient;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    instruction::{AccountMeta, Instruction},
    message::Message,
    pubkey::Pubkey,
    system_program,
    transaction::Transaction,
};
use sqlx::PgPool;
use std::time::Duration;

use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::ID as associated_token_program_id;
use spl_token::ID as token_program_id;

use crate::models::premarket::{BuildFinishTxParams, BuiltTx};

use super::constants::{constants, FINISH_METHOD_NAME};
use super::env::{program_id_for, read_revelcy_auth, rpc_url};
use super::utils::{anchor_sighash_global, get_valid_latest_blockhash, parse_privkey_64, pda};

use crate::storage::signing_keys::get_mint_signing_keypair_by_premarket;
use solana_sdk::signature::Keypair;

pub async fn get_mint_kp(pool: &PgPool, premarket: Pubkey) -> Result<Keypair> {
    let pair = get_mint_signing_keypair_by_premarket(pool, &premarket.to_string())
        .await
        .context("signing_keys: mint key not found for this premarket")?
        .ok_or_else(|| anyhow::anyhow!("mint key not found for premarket {}", premarket))?;

    let mint_bytes = parse_privkey_64(&pair.priv_key).context("mint priv_key parse failed")?;
    let mint_kp =
        Keypair::from_bytes(&mint_bytes).context("mint priv_key: invalid keypair bytes")?;
    Ok(mint_kp)
}

pub async fn build_finish_premarket_tx_unsigned(
    pool: &PgPool,
    params: BuildFinishTxParams,
) -> Result<BuiltTx> {
    let program_id = program_id_for(params.network);
    let rpc = AsyncRpcClient::new_with_timeout(rpc_url(params.network), Duration::from_secs(15));

    let (
        mint_auth,
        pump_fun_program_id,
        pumpfun_global,
        metaplex_program,
        event_auth,
        fee_recipient,
        rent_sysvar,
        global_volume_accum,
        fee_program,
    ) = constants(params.network);

    // mint key из БД
    let mint_kp = get_mint_kp(pool, params.premarket).await?;
    let mint_pub = mint_kp.pubkey();

    // PDAs/ATAs
    let (bonding_curve, _) = pda(&pump_fun_program_id, &[b"bonding-curve", mint_pub.as_ref()]);
    let bonding_curve_ata = get_associated_token_address(&bonding_curve, &mint_pub);
    let (metadata, _) = pda(
        &metaplex_program,
        &[b"metadata", metaplex_program.as_ref(), mint_pub.as_ref()],
    );

    let revelcy = read_revelcy_auth(params.network);
    let revelcy_pub = revelcy.pubkey();
    let associated_user_ata = get_associated_token_address(&revelcy_pub, &mint_pub);

    let (creator_vault, _) = pda(
        &pump_fun_program_id,
        &[b"creator-vault", params.user.as_ref()],
    );
    let (user_volume_accum, _) = pda(
        &pump_fun_program_id,
        &[b"user_volume_accumulator", revelcy_pub.as_ref()],
    );

    let seed1: &[u8] = b"fee_config";
    let seed2: [u8; 32] = [
        1, 86, 224, 246, 147, 102, 90, 207, 68, 219, 21, 104, 191, 23, 91, 170, 81, 137, 203, 151,
        245, 210, 255, 59, 101, 93, 43, 182, 253, 109, 24, 176,
    ];
    let (fee_config, _) = Pubkey::find_program_address(&[seed1, &seed2], &fee_program);

    // discriminator only
    let mut data = Vec::with_capacity(8);
    data.extend_from_slice(&anchor_sighash_global(FINISH_METHOD_NAME));

    // IDL порядок
    let accounts = vec![
        AccountMeta::new(revelcy_pub, true),                  // 1
        AccountMeta::new(params.premarket, false),            // 2
        AccountMeta::new(mint_pub, true),                     // 3
        AccountMeta::new_readonly(mint_auth, false),          // 4
        AccountMeta::new(bonding_curve, false),               // 5
        AccountMeta::new(bonding_curve_ata, false),           // 6
        AccountMeta::new(pumpfun_global, false),              // 7
        AccountMeta::new(metaplex_program, false), // 8 (как у тебя; хотя обычно readonly)
        AccountMeta::new(metadata, false),         // 9
        AccountMeta::new(params.user, true),       // 10
        AccountMeta::new_readonly(system_program::ID, false), // 11
        AccountMeta::new_readonly(token_program_id, false), // 12
        AccountMeta::new_readonly(associated_token_program_id, false), // 13
        AccountMeta::new_readonly(rent_sysvar, false), // 14
        AccountMeta::new(event_auth, false),       // 15
        AccountMeta::new_readonly(pump_fun_program_id, false), // 16
        AccountMeta::new(fee_recipient, false),    // 17
        AccountMeta::new(associated_user_ata, false), // 18
        AccountMeta::new(creator_vault, false),    // 19
        AccountMeta::new(global_volume_accum, false), // 20
        AccountMeta::new(user_volume_accum, false), // 21
        AccountMeta::new(fee_config, false),       // 22
        AccountMeta::new(fee_program, false),      // 23
    ];

    let ix_finish = Instruction {
        program_id,
        accounts,
        data,
    };
    let ix_compute = ComputeBudgetInstruction::set_compute_unit_limit(400_000);

    let blockhash = get_valid_latest_blockhash(&rpc, 50)
        .await
        .context("get_latest_blockhash failed")?;

    let msg = Message::new(&[ix_compute, ix_finish], Some(&params.user));
    let mut tx = Transaction::new_unsigned(msg);
    tx.message.recent_blockhash = blockhash;

    let raw = bincode::serialize(&tx).context("serialize tx failed")?;
    let tx_b64 = BASE64.encode(raw);

    Ok(BuiltTx {
        tx_base64: tx_b64,
        premarket_pda: params.premarket,
    })
}
