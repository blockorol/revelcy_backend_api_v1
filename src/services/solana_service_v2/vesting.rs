
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