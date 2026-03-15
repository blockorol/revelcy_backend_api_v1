use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use bincode;
use borsh::{BorshDeserialize, BorshSerialize};
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

use crate::models::premarket::{BuiltTx, SolanaNetwork};

use super::constants::UPDATE_PREMARKET_DATA_METHOD_NAME;
use super::env::{program_id_for, read_revelcy_auth, rpc_url};
use super::utils::{anchor_sighash_global, get_valid_latest_blockhash};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct UpdatePremarketDataArgs {
    pub end_timestamp: Option<i64>,
    pub end_timestamp_updated: Option<bool>,
    pub goal_sol: Option<u64>,
    pub max_sol: Option<u64>,
    pub mint: Option<Pubkey>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub uri: Option<String>,
    pub creator: Option<Pubkey>,
}

pub async fn build_update_premarket_data_tx_unsigned(
    network: SolanaNetwork,
    user: Pubkey,
    premarket: Pubkey,
    args: UpdatePremarketDataArgs,
) -> Result<BuiltTx> {
    let program_id = program_id_for(network);
    let rpc = AsyncRpcClient::new_with_timeout(rpc_url(network), Duration::from_secs(15));

    let revelcy_auth = read_revelcy_auth(network);

    let accounts = vec![
        AccountMeta::new(revelcy_auth.pubkey(), true),
        AccountMeta::new(user, true),
        AccountMeta::new(premarket, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];

    // discriminator + borsh(args)
    let mut data = Vec::with_capacity(8 + 256);
    data.extend_from_slice(&anchor_sighash_global(UPDATE_PREMARKET_DATA_METHOD_NAME));
    args.serialize(&mut data)
        .context("borsh serialize UpdatePremarketDataArgs failed")?;

    let ix = Instruction {
        program_id,
        accounts,
        data,
    };

    let blockhash = get_valid_latest_blockhash(&rpc, 50)
        .await
        .context("get_latest_blockhash failed")?;

    let msg = Message::new(&[ix], Some(&user));
    let mut tx = Transaction::new_unsigned(msg);
    tx.message.recent_blockhash = blockhash;

    let raw = bincode::serialize(&tx).context("serialize tx failed")?;
    let tx_b64 = BASE64.encode(raw);

    Ok(BuiltTx {
        tx_base64: tx_b64,
        premarket_pda: Some(premarket),
    })
}
