// services/background_finaliser.rs
use std::{future::Future, pin::Pin, time::Duration};
use solana_client::nonblocking::rpc_client::RpcClient as AsyncRpcClient;
use solana_sdk::signature::Signature;
use crate::models::premarket::SolanaNetwork;

pub type UpdateFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;
pub type UpdateFn = Box<dyn FnOnce(Signature, AsyncRpcClient) -> UpdateFuture + Send + 'static>;

pub fn background_finalize_action(
    network: SolanaNetwork,
    sig: Signature,
    timeout: Duration,
    poll_every: Duration,
    update: UpdateFn,
) {
    tokio::spawn(async move {
        let rpc = crate::services::solana_service_v2::make_async_rpc_client(network);
        match crate::services::solana_service_v2::wait_for_finalized_with_client(
            &rpc,
            &sig,
            timeout,
            poll_every,
        )
        .await
        {
            Ok(()) => {
                (update)(sig, rpc).await;
            }
            Err(e) => {
                eprintln!("wait_for_finalized failed: sig={} err={:#}", sig, e);
            }
        }
    });
}
