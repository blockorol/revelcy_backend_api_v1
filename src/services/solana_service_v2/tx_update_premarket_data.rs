use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use bincode;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::nonblocking::rpc_client::RpcClient as AsyncRpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    message::Message,
    signature::Signer,
    pubkey::Pubkey,
    system_program,
    transaction::Transaction,
};
use std::{str::FromStr, time::Duration};

use crate::models::premarket::{BuiltTx, SolanaNetwork, UpdatePremarketDataParams};

use super::constants::UPDATE_PREMARKET_DATA_METHOD_NAME;
use super::env::{program_id_for, read_revelcy_auth, rpc_url};
use super::utils::{anchor_sighash_global, get_valid_latest_blockhash};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct UpdatePremarketDataArgs {
    pub end_timestamp: Option<i64>,
    pub end_timestamp_updated: Option<bool>,
    pub goal_sol: Option<u64>,
    pub max_sol: Option<u64>,
    pub mint: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub uri: Option<String>,
    pub creator: Option<String>,
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

pub async fn update_premarket_data_tx_unsigned(
    params: UpdatePremarketDataParams,
) -> Result<BuiltTx> {
    let network = SolanaNetwork::try_from(params.network.as_str())
        .map_err(|e| anyhow::anyhow!("invalid network: {e}"))?;

    let premarket =
        Pubkey::from_str(&params.premarket_account).context("invalid premarket_account pubkey")?;
    let user = Pubkey::from_str(&params.user_pubkey).context("invalid user_pubkey")?;

    let args = UpdatePremarketDataArgs {
        end_timestamp: params.end_timestamp,
        end_timestamp_updated: params.end_timestamp_updated,
        goal_sol: params.goal_sol,
        max_sol: params.max_sol,
        mint: params.mint,
        name: params.name,
        symbol: params.symbol,
        uri: params.uri,
        creator: params.creator,
    };

    build_update_premarket_data_tx_unsigned(network, user, premarket, args).await
}
