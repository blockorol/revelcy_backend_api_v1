use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use bincode::deserialize;
use crate::models::premarket::SolanaNetwork;
use crate::services::solana_service_v2::env::rpc_url;
use solana_client::nonblocking::rpc_client::RpcClient as AsyncRpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::signature::Signature;
use solana_sdk::transaction::Transaction;
use solana_transaction_status::option_serializer::OptionSerializer;
use solana_transaction_status::{UiTransactionEncoding, UiTransactionTokenBalance};
use std::str::FromStr;
use tokio::time::{sleep, Duration, Instant};


pub async fn send_signed_tx_base64(
    network: SolanaNetwork,
    signed_tx_base64: &str,
) -> Result<Signature> {
    let rpc = AsyncRpcClient::new_with_timeout(rpc_url(network), Duration::from_secs(15));

    let raw = BASE64
        .decode(signed_tx_base64.trim())
        .context("invalid base64 for signed tx")?;

    let transaction: Transaction = deserialize(&raw)
        .context("failed to deserialize Transaction")?;

    let sig = rpc
        .send_transaction_with_config(
            &transaction,
            RpcSendTransactionConfig {
                skip_preflight: false,
                preflight_commitment: Some(CommitmentLevel::Processed),
                max_retries: Some(5),
                min_context_slot: None,
                encoding: None,
            },
        )
        .await
        .context("send_transaction_with_config failed")?;

    Ok(sig)
}

pub async fn wait_for_finalized(    
    network: SolanaNetwork,
    sig: &Signature,
    timeout: Duration,
    poll_every: Duration,
) -> Result<()> {
    let rpc: AsyncRpcClient = AsyncRpcClient::new_with_timeout(rpc_url(network), Duration::from_secs(15));
    let started = Instant::now();

    loop {
        if started.elapsed() > timeout {
            return Err(anyhow!("timeout waiting for tx to finalize: {}", sig));
        }

        let st = rpc
            .get_signature_status_with_commitment(sig, CommitmentConfig::finalized())
            .await
            .context("get_signature_status_with_commitment failed")?;

        if let Some(status) = st {
            if let Err(err) = status {
                return Err(anyhow!("transaction failed: {} err={:?}", sig, err));
            }
            return Ok(());
        }

        sleep(poll_every).await;
    }
}

pub async fn wait_for_confirmed( 
    network: SolanaNetwork,
    sig: &Signature,
    timeout: Duration,
    poll_every: Duration,
) -> Result<()> {
    let rpc: AsyncRpcClient = AsyncRpcClient::new_with_timeout(rpc_url(network), Duration::from_secs(15));
    let started = Instant::now();
    loop {
        if started.elapsed() > timeout {
            return Err(anyhow!("timeout waiting for confirmed: {}", sig));
        }

        let st = rpc
            .get_signature_status_with_commitment(sig, CommitmentConfig::confirmed())
            .await
            .context("get_signature_status_with_commitment(confirmed) failed")?;
        if let Some(status) = st {
            if let Err(err) = status {
                return Err(anyhow!("tx failed before confirmed: {} err={:?}", sig, err));
            }
            return Ok(());
        }

        sleep(poll_every).await;
    }
}

fn token_amount_u128(amount_str: &str) -> u128 {
    amount_str.parse::<u128>().unwrap_or(0)
}

fn find_amount_for_mint_owner(
    balances: &[UiTransactionTokenBalance],
    mint: &str,
    owner: &str,
) -> u128 {
    balances
        .iter()
        .find(|b| {
            b.mint == mint
                && matches!(&b.owner, OptionSerializer::Some(v) if v == owner)
        })
        .map(|b| token_amount_u128(&b.ui_token_amount.amount))
        .unwrap_or(0)
}

pub async fn get_spl_token_delta(
    network: SolanaNetwork,
    signature: &Signature,
    mint: &str,
    owner: &str,
) -> Result<i128> {
    println!(
        "[get_spl_token_delta] start network={:?} sig={} mint={} owner={}",
        network, signature, mint, owner
    );

    let rpc = AsyncRpcClient::new_with_timeout(rpc_url(network), Duration::from_secs(15));

    let tx = rpc
        .get_transaction_with_config(
            signature,
            RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::JsonParsed),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(0),
            },
        )
        .await
        .context("get_transaction_with_config failed")?;
    println!(
        "[get_spl_token_delta] tx loaded: slot={} block_time={:?}",
        tx.slot, tx.block_time
    );

    let meta = tx
        .transaction
        .meta
        .as_ref()
        .ok_or_else(|| anyhow!("No meta in transaction"))?;
    println!(
        "[get_spl_token_delta] meta present: err={:?}",
        meta.err
    );

    let pre = match &meta.pre_token_balances {
        OptionSerializer::Some(v) => v.as_slice(),
        _ => &[],
    };
    let post = match &meta.post_token_balances {
        OptionSerializer::Some(v) => v.as_slice(),
        _ => &[],
    };
    println!(
        "[get_spl_token_delta] balances: pre_count={} post_count={}",
        pre.len(),
        post.len()
    );

    for (i, b) in pre.iter().enumerate() {
        println!(
            "[get_spl_token_delta] pre[{}]: mint={} owner={:?} amount={}",
            i, b.mint, b.owner, b.ui_token_amount.amount
        );
    }
    for (i, b) in post.iter().enumerate() {
        println!(
            "[get_spl_token_delta] post[{}]: mint={} owner={:?} amount={}",
            i, b.mint, b.owner, b.ui_token_amount.amount
        );
    }

    let pre_amt = find_amount_for_mint_owner(pre, mint, owner);
    let post_amt = find_amount_for_mint_owner(post, mint, owner);
    let delta = post_amt as i128 - pre_amt as i128;

    println!(
        "[get_spl_token_delta] result: pre_amt={} post_amt={} delta={}",
        pre_amt, post_amt, delta
    );

    Ok(delta)
}
