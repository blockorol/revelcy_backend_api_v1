use crate::config;
use crate::models::premarket::SolanaNetwork;
use solana_client::nonblocking::rpc_client::RpcClient as AsyncRpcClient;
use std::env::VarError;
use std::time::Duration;

pub const DEFAULT_SOLANA_RPC_TIMEOUT: Duration = Duration::from_secs(15);

pub fn rpc_url_for_network(network: SolanaNetwork) -> String {
    match network {
        SolanaNetwork::Devnet => config::get_solana_devnet_rpc(),
        SolanaNetwork::MainnetBeta => config::get_solana_mainnet_rpc(),
    }
}

pub fn make_async_rpc_client(network: SolanaNetwork) -> AsyncRpcClient {
    make_async_rpc_client_from_url(rpc_url_for_network(network))
}

pub fn make_default_async_rpc_client() -> Result<AsyncRpcClient, VarError> {
    config::get_solana_rpc().map(make_async_rpc_client_from_url)
}

pub fn make_async_rpc_client_from_url(rpc_url: String) -> AsyncRpcClient {
    AsyncRpcClient::new_with_timeout(rpc_url, DEFAULT_SOLANA_RPC_TIMEOUT)
}
