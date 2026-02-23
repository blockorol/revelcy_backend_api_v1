
use anyhow::{anyhow, Result};
use solana_client::nonblocking::rpc_client::RpcClient as AsyncRpcClient;
use solana_sdk::pubkey::Pubkey;
use super::env::program_id_for;
use crate::models::premarket::SolanaNetwork;


pub fn generate_vesting_pda(
    network: SolanaNetwork,
    mint: Pubkey,
) -> Pubkey {
    let program_id = program_id_for(network);

    let vesting_seed = b"vesting";
    let (vesting_pda, _bump) = Pubkey::find_program_address(
        &[vesting_seed, mint.as_ref()],
        &program_id,
    );

    vesting_pda
}

#[derive(Debug, Clone)]
pub struct OnchainVestingUser {
    pub user_pubkey: Pubkey,
    pub tokens_total: u64,
    pub tokens_claimed: u64,
}

#[derive(Debug, Clone)]
pub struct OnchainVestingData {
    pub start_timestamp: i64,
    pub end_timestamp: i64,
    pub init_unlock: u64,
    pub users: Vec<OnchainVestingUser>,
}

pub async fn get_vesting_account_data_with_client(
    client: &AsyncRpcClient,
    vesting_account: Pubkey,
) -> Result<OnchainVestingData> {
    let account = client.get_account(&vesting_account).await?;
    parse_vesting_account_data(&account.data)
}

fn parse_vesting_account_data(data: &[u8]) -> Result<OnchainVestingData> {
    if data.len() < 8 + 8 + 8 + 8 + 4 {
        return Err(anyhow!("vesting account data too short"));
    }

    // Anchor discriminator
    let mut offset = 8usize;

    let start_timestamp = i64::from_le_bytes(
        data[offset..offset + 8]
            .try_into()
            .map_err(|_| anyhow!("invalid start_timestamp bytes"))?,
    );
    offset += 8;

    let end_timestamp = i64::from_le_bytes(
        data[offset..offset + 8]
            .try_into()
            .map_err(|_| anyhow!("invalid end_timestamp bytes"))?,
    );
    offset += 8;

    let init_unlock = u64::from_le_bytes(
        data[offset..offset + 8]
            .try_into()
            .map_err(|_| anyhow!("invalid init_unlock bytes"))?,
    );
    offset += 8;

    let users_len = u32::from_le_bytes(
        data[offset..offset + 4]
            .try_into()
            .map_err(|_| anyhow!("invalid users_len bytes"))?,
    ) as usize;
    offset += 4;

    let mut users = Vec::with_capacity(users_len);
    for _ in 0..users_len {
        if data.len() < offset + 32 + 8 + 8 {
            return Err(anyhow!("vesting users section is truncated"));
        }

        let user_pubkey = Pubkey::new_from_array(
            data[offset..offset + 32]
                .try_into()
                .map_err(|_| anyhow!("invalid user_pubkey bytes"))?,
        );
        offset += 32;

        let tokens_total = u64::from_le_bytes(
            data[offset..offset + 8]
                .try_into()
                .map_err(|_| anyhow!("invalid tokens_total bytes"))?,
        );
        offset += 8;

        let tokens_claimed = u64::from_le_bytes(
            data[offset..offset + 8]
                .try_into()
                .map_err(|_| anyhow!("invalid tokens_claimed bytes"))?,
        );
        offset += 8;

        users.push(OnchainVestingUser {
            user_pubkey,
            tokens_total,
            tokens_claimed,
        });
    }

    Ok(OnchainVestingData {
        start_timestamp,
        end_timestamp,
        init_unlock,
        users,
    })
}
