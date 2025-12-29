use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use bincode;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::nonblocking::rpc_client::RpcClient as AsyncRpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    message::Message,
    pubkey::Pubkey,
    transaction::Transaction,
};
use std::time::Duration;

use crate::models::premarket::{BuildKillTxParams, BuiltTx, SolanaNetwork};

use super::constants::KILL_METHOD_NAME;
use super::env::{program_id_for, read_revelcy_auth, rpc_url};
use super::utils::{
    anchor_sighash_global, get_valid_latest_blockhash,
};

#[derive(Debug, Clone)]
pub struct ParsedKillPremarketTx {
    pub revelcy_auth: Pubkey,
    pub user: Pubkey,
    pub premarket: Pubkey,
    pub params: BuildKillTxParams,
}

pub async fn build_kill_premarket_tx_unsigned(
    _pool: &PgPool,
    params: BuildKillTxParams,
) -> Result<BuiltTx> {
    let program_id = program_id_for(params.network);
    let client = AsyncRpcClient::new_with_timeout(rpc_url(params.network), Duration::from_secs(15));
    let revelcy = read_revelcy_auth(params.network);
    let revelcy_pub = revelcy.pubkey();
    let premarket_account = params.premarket;
    let user = params.user;
    let all_entered_users = params.users;

    let mut accounts = vec![
        AccountMeta::new(revelcy_pub, true),
        AccountMeta::new(premarket_account, false),
        AccountMeta::new(user, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
    ];

    for user in all_entered_users {
        accounts.push(AccountMeta::new(Pubkey::from_str(&user).unwrap(), false));
    }

    let _discriminator: [u8; 8] = [
        10,
        112,
        216,
        238,
        253,
        26,
        122,
        160
    ];

    let mut data = Vec::with_capacity(8);
    //data.extend_from_slice(&discriminator);
    data.extend_from_slice(&anchor_sighash_global(KILL_METHOD_NAME));

    let ix = Instruction { program_id, accounts, data };
    let blockhash = get_valid_latest_blockhash(&client, 50)
        .await
        .context("get_latest_blockhash failed")?;

    let msg = Message::new(&[ix], Some(&params.user));
    let mut tx = Transaction::new_unsigned(msg);

    tx.message.recent_blockhash = blockhash;

    let raw = bincode::serialize(&tx).context("serialize tx failed")?;
    let tx_b64 = BASE64.encode(raw);

    Ok(BuiltTx {
        tx_base64: tx_b64,
        premarket_pda: params.premarket,
    })
}
