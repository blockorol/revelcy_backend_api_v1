use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use bincode::deserialize;
use crate::models::premarket::SolanaNetwork;
use crate::services::solana_service_v2::env::rpc_url;
use solana_client::nonblocking::rpc_client::RpcClient as AsyncRpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::signature::Signature;
use solana_sdk::transaction::Transaction;
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
