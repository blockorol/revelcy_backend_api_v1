use anyhow::Result;
use solana_sdk::pubkey::Pubkey;

use crate::models::premarket::{BuiltTx, SolanaNetwork};

use super::update_premarket_data_tx::{build_update_premarket_data_tx_unsigned, UpdatePremarketDataArgs};

pub async fn build_extend_premarket_tx_unsigned(
    network: SolanaNetwork,
    user: Pubkey,
    premarket: Pubkey,
    new_deadline: i64,
) -> Result<BuiltTx> {
    let args = UpdatePremarketDataArgs {
        end_timestamp: Some(new_deadline),
        end_timestamp_updated: Some(true),
        goal_sol: None,
        max_sol: None,
        mint: None,
        name: None,
        symbol: None,
        uri: None,
        creator: None,
    };

    build_update_premarket_data_tx_unsigned(network, user, premarket, args).await
}
