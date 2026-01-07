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
use crate::storage::signing_keys::{
    delete_signing_key_by_pubkey, get_unused_signing_key, insert_mint_signing_key,
};

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

pub async fn build_create_premarket_tx_unsigned(
    pool: &PgPool,
    params: BuildPremarketTxParams,
) -> Result<BuiltTxCreation> {
    let program_id = program_id_for(params.network);
    let rpc = AsyncRpcClient::new_with_timeout(rpc_url(params.network), Duration::from_secs(15));

    let mint = if let Some(pair) = get_unused_signing_key(pool).await? {
        let bytes =
            parse_privkey_64(&pair.priv_key).context("signing_keys.priv_key parse failed")?;
        Keypair::from_bytes(&bytes).context("invalid keypair bytes in signing_keys")?
    } else {
        Keypair::new()
    };

    let mint_pub = mint.pubkey().to_string();
    delete_signing_key_by_pubkey(pool, &mint_pub).await?; // should be looked instead of "delete"

    let revelcy = read_revelcy_auth(params.network);
    let revelcy_pub = revelcy.pubkey();
    let (premarket_pda, _bump) =
        Pubkey::find_program_address(&[revelcy_pub.as_ref(), mint.pubkey().as_ref()], &program_id);

    let priv_b58 = bs58::encode(mint.to_bytes()).into_string();
    insert_mint_signing_key(pool, &premarket_pda.to_string(), &mint_pub, &priv_b58)
        .await
        .context("failed to insert mint key into signing_keys")?;

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
            AccountMeta::new_readonly(mint.pubkey(), false),
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

    if let Err(ref e) = result {
        if let Err(clean_err) = delete_signing_key_by_pubkey(pool, &mint_pub).await {
            eprintln!(
                "cleanup: failed to delete signing_key for pub_key {}: {clean_err:?} (root error: {e:?})",
                mint_pub
            );
        }
    }

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

    if ix.accounts.len() < 4 {
        return Err(anyhow!("instruction accounts too short (<4)"));
    }

    let revelcy_auth = resolve_account(msg, ix.accounts[0] as usize)?;
    let premarket_pda = resolve_account(msg, ix.accounts[1] as usize)?;
    let mint = resolve_account(msg, ix.accounts[2] as usize)?;
    let user = resolve_account(msg, ix.accounts[3] as usize)?;

    let params = BuildPremarketTxParams {
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
