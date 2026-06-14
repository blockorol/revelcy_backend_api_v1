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
    signature::{Keypair, Signer},
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
    let (bonding_curve_v2, _) = pda(
        &pump_fun_program_id,
        &[b"bonding-curve-v2", mint_pub.as_ref()],
    );
    let bonding_curve_ata = get_associated_token_address(&bonding_curve, &mint_pub);
    let (metadata, _) = pda(
        &metaplex_program,
        &[b"metadata", metaplex_program.as_ref(), mint_pub.as_ref()],
    );

    let revelcy = read_revelcy_auth(params.network);
    let revelcy_pub = revelcy.pubkey();
    let revelcy_ata = get_associated_token_address(&revelcy_pub, &mint_pub);

    // Vesting account PDA and ATA
    let (vesting_account, _) = pda(&program_id, &[b"vesting", mint_pub.as_ref()]);
    let vesting_ata = get_associated_token_address(&vesting_account, &mint_pub);

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

    println!("finish_premarket accounts:");
    println!("  program_id: {}", program_id);
    println!("  revelcy_auth: {}", revelcy_pub);
    println!("  revelcy_ata: {}", revelcy_ata);
    println!("  premarket_account: {}", params.premarket);
    println!("  token_mint: {}", mint_pub);
    println!("  mint_auth: {}", mint_auth);
    println!("  bonding_curve: {}", bonding_curve);
    println!("  bonding_curve_v2: {}", bonding_curve_v2);
    println!("  bonding_curve_ata: {}", bonding_curve_ata);
    println!("  global: {}", pumpfun_global);
    println!("  mpl_token_metadata: {}", metaplex_program);
    println!("  metadata: {}", metadata);
    println!("  user: {}", params.user);
    println!("  vesting_account: {}", vesting_account);
    println!("  vesting_ata: {}", vesting_ata);
    println!("  system_program: {}", system_program::ID);
    println!("  token_program: {}", token_program_id);
    println!(
        "  associated_token_program: {}",
        associated_token_program_id
    );
    println!("  rent: {}", rent_sysvar);
    println!("  event_auth: {}", event_auth);
    println!("  pump_fun_program_id: {}", pump_fun_program_id);
    println!("  fee_recipient: {}", fee_recipient);
    println!("  associated_user: {}", associated_user_ata);
    println!("  creator_vault: {}", creator_vault);
    println!("  global_volume_accumulator: {}", global_volume_accum);
    println!("  user_volume_accumulator: {}", user_volume_accum);
    println!("  fee_config: {}", fee_config);
    println!("  fee_program: {}", fee_program);

    // Serialize discriminator + args
    let mut data = Vec::with_capacity(32);
    data.extend_from_slice(&anchor_sighash_global(FINISH_METHOD_NAME));

    // Serialize FinishPremarketArgs: timestamp_start (i64), timestamp_end (i64), init_unlock (u64)
    data.extend_from_slice(&params.timestamp_start.to_le_bytes());
    data.extend_from_slice(&params.timestamp_end.to_le_bytes());
    data.extend_from_slice(&params.init_unlock.to_le_bytes());

    // IDL порядок (based on finish_premarket instruction)
    let accounts = vec![
        AccountMeta::new(revelcy_pub, true),         // 1. revelcy_auth
        AccountMeta::new(revelcy_ata, false),        // 2. revelcy_ata
        AccountMeta::new(vesting_account, false),    // 3. vesting_account (PDA)
        AccountMeta::new(vesting_ata, false),        // 4. vesting_ata
        AccountMeta::new(params.premarket, false),   // 5. premarket_account
        AccountMeta::new(mint_pub, true),            // 6. token_mint (writable, signer)
        AccountMeta::new_readonly(mint_auth, false), // 7. mint_auth
        AccountMeta::new(bonding_curve, false),      // 8. bonding_curve
        AccountMeta::new(bonding_curve_ata, false),  // 9. bonding_curve_ata
        AccountMeta::new(pumpfun_global, false),     // 10. global
        AccountMeta::new(metaplex_program, false),   // 11. mpl_token_metadata
        AccountMeta::new(metadata, false),           // 12. metadata
        AccountMeta::new(params.user, true),         // 13. user (writable, signer)
        AccountMeta::new_readonly(system_program::ID, false), // 14. system_program
        AccountMeta::new_readonly(token_program_id, false), // 15. token_program
        AccountMeta::new_readonly(associated_token_program_id, false), // 16. associated_token_program
        AccountMeta::new_readonly(rent_sysvar, false),                 // 17. rent
        AccountMeta::new(event_auth, false),                           // 18. event_auth
        AccountMeta::new_readonly(pump_fun_program_id, false),         // 19. pump_fun_program_id
        AccountMeta::new(fee_recipient, false),                        // 20. fee_recipient
        AccountMeta::new(associated_user_ata, false),                  // 21. associated_user
        AccountMeta::new(creator_vault, false),                        // 22. creator_vault
        AccountMeta::new(global_volume_accum, false), // 23. global_volume_accumulator
        AccountMeta::new(user_volume_accum, false),   // 24. user_volume_accumulator
        AccountMeta::new(fee_config, false),          // 25. fee_config
        AccountMeta::new(fee_program, false),         // 26. fee_program
        AccountMeta::new_readonly(bonding_curve_v2, false), // 27. bonding_curve_v2
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
        premarket_pda: Some(params.premarket),
    })
}
