use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use bincode;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::nonblocking::rpc_client::RpcClient as AsyncRpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    message::Message,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};
use sqlx::PgPool;
use std::time::Duration;

use crate::models::premarket::{BuildPremarketTxParams, BuiltTxCreation, SolanaNetwork};

use super::constants::CREATE_METHOD_NAME;
use super::env::{program_id_for, read_revelcy_auth, rpc_url};
use super::utils::{
    anchor_sighash_global, find_anchor_instruction, get_valid_latest_blockhash, parse_privkey_64,
    resolve_account,
};

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
struct CreatePremarketArgsBorsh {
    pub end_timestamp: i64,
    pub goal_sol: u64,
    pub max_sol: u64,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub amount_in_lamports: u64,
}

#[derive(Debug, Clone)]
pub struct ParsedCreatePremarketTx {
    pub revelcy_auth: Pubkey,
    pub premarket_pda: Pubkey,
    pub mint: Pubkey,
    pub user: Pubkey,
    pub params: BuildPremarketTxParams,
}

pub async fn generate_premarket_pda(
    network: SolanaNetwork,
    mint_priv: &str,
) -> Result<Pubkey> {
    let bytes= parse_privkey_64(mint_priv)
        .context("mint_priv parse failed")?;

    let mint = Keypair::from_bytes(&bytes)
        .context("invalid mint keypair bytes")?;

    let program_id = program_id_for(network);
    let revelcy = read_revelcy_auth(network);
    let revelcy_pub = revelcy.pubkey();

    let (premarket_pda, _bump) = Pubkey::find_program_address(
        &[revelcy_pub.as_ref(), mint.pubkey().as_ref()],
        &program_id,
    );

    Ok(premarket_pda)
}

pub async fn build_create_premarket_tx_unsigned(
    _pool: &PgPool,
    params: BuildPremarketTxParams,
) -> Result<BuiltTxCreation> {
    let program_id = program_id_for(params.network);
    let rpc = AsyncRpcClient::new_with_timeout(rpc_url(params.network), Duration::from_secs(15));

    let revelcy = read_revelcy_auth(params.network);
    let revelcy_pub = revelcy.pubkey();
    let mint_pub = params.mint.to_string();
    let premarket_pda = params.premarket_pda.clone();

    let result: Result<BuiltTxCreation> = async {
        let mut data = Vec::with_capacity(8 + 128);
        data.extend_from_slice(&anchor_sighash_global(CREATE_METHOD_NAME));

        CreatePremarketArgsBorsh {
            end_timestamp: params.deadline,
            goal_sol: params.goal,
            max_sol: params.max,
            name: params.name,
            symbol: params.symbol,
            uri: params.uri,
            amount_in_lamports: params.creator_allocate,
        }
        .serialize(&mut data)
        .context("borsh serialize of CreatePremarketArgs failed")?;

        let accounts = vec![
            AccountMeta::new(revelcy_pub, true),
            AccountMeta::new(premarket_pda, false),
            AccountMeta::new_readonly(params.mint, false),
            AccountMeta::new(params.user, true),
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

        Ok(BuiltTxCreation {
            mint_address: mint_pub.clone(),
            tx_base64: tx_b64,
            premarket_pda,
        })
    }
    .await;

    result
}

pub fn parse_create_premarket_tx_from_base64(
    tx_b64: &str,
    network: SolanaNetwork,
) -> Result<ParsedCreatePremarketTx> {
    let program_id = program_id_for(network);
    let expected_sighash = anchor_sighash_global(CREATE_METHOD_NAME);

    let raw = BASE64.decode(tx_b64).context("tx_base64 decode failed")?;

    let tx: Transaction =
        bincode::deserialize(&raw).context("bincode deserialize(Transaction) failed")?;
    let msg: &Message = &tx.message;

    let ix = find_anchor_instruction(msg, &program_id, &expected_sighash)
        .context("create_premarket instruction not found")?;

    if ix.data.len() < 8 {
        return Err(anyhow!("instruction data too short (<8)"));
    }

    let args = CreatePremarketArgsBorsh::try_from_slice(&ix.data[8..])
        .context("borsh decode CreatePremarketArgs failed")?;

    if ix.accounts.len() < 5 {
        return Err(anyhow!("instruction accounts too short (<5)"));
    }

    let revelcy_auth = resolve_account(msg, ix.accounts[0] as usize)?;
    let premarket_pda = resolve_account(msg, ix.accounts[1] as usize)?;
    let mint = resolve_account(msg, ix.accounts[2] as usize)?;
    let user = resolve_account(msg, ix.accounts[3] as usize)?;
    let system_program_key = resolve_account(msg, ix.accounts[4] as usize)?;

    if system_program_key != system_program::ID {
        return Err(anyhow!("invalid system_program account"));
    }

    let params = BuildPremarketTxParams {
        mint,
        premarket_pda,
        network,
        user,
        deadline: args.end_timestamp,
        goal: args.goal_sol,
        max: args.max_sol,
        creator_allocate: args.amount_in_lamports,
        name: args.name,
        symbol: args.symbol,
        uri: args.uri,
    };

    Ok(ParsedCreatePremarketTx {
        revelcy_auth,
        premarket_pda,
        mint,
        user,
        params,
    })
}
