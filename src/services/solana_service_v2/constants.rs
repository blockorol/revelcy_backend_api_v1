use solana_sdk::pubkey::Pubkey;
use crate::models::premarket::SolanaNetwork;
use super::utils::{pk};



pub const CREATE_METHOD_NAME: &str = "create_premarket";
pub const JOIN_METHOD_NAME: &str = "join_to_premarket";
pub const OUT_METHOD_NAME: &str = "out_of_premarket";
pub const FINISH_METHOD_NAME: &str = "finish_premarket";
pub const UPDATE_PREMARKET_DATA_METHOD_NAME: &str = "update_premarket_data";
pub const CLAIM_TOKENS_METHOD_NAME: &str = "claim_tokens";

pub fn constants(network: SolanaNetwork) -> (
    Pubkey, // MINT_AUTH
    Pubkey, // PUMP_FUN_PROGRAM_ID
    Pubkey, // PUMPFUN_GLOBAL
    Pubkey, // METAPLEX_PROGRAM
    Pubkey, // PUMPFUN_EVENT_AUTH
    Pubkey, // FEE_RECIPIENT
    Pubkey, // RENT
    Pubkey, // GLOBAL_VOLUME_ACCUMULATOR
    Pubkey, // FEE_PROGRAM
) {
    match network {
        SolanaNetwork::Devnet => (
            pk("TSLvdd1pWpHVjahSpsvCXUbgwsL3JAcvokwaKt1eokM"),
            pk("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P"),
            pk("4wTV1YmiEkRvAtNtsSGPtUrqRYQMe5SKy2uB4Jjaxnjf"),
            pk("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"),
            pk("Ce6TQqeHC9p8KetsN6JsjHK7UTZk7nasjjnr7XxXp9F1"),
            pk("68yFSZxzLWJXkxxRGydZ63C6mHx1NLEDWmwN9Lb5yySg"),
            pk("SysvarRent111111111111111111111111111111111"),
            pk("Hq2wp8uJ9jCPsYgNHex8RtqdvMPfVGoYwjvF1ATiwn2Y"),
            pk("pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ"), // FEE_PROGRAM (placeholder)
        ),
        SolanaNetwork::MainnetBeta => (
            pk("TSLvdd1pWpHVjahSpsvCXUbgwsL3JAcvokwaKt1eokM"),
            pk("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P"),
            pk("4wTV1YmiEkRvAtNtsSGPtUrqRYQMe5SKy2uB4Jjaxnjf"),
            pk("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"),
            pk("Ce6TQqeHC9p8KetsN6JsjHK7UTZk7nasjjnr7XxXp9F1"),
            pk("9rPYyANsfQZw3DnDmKE3YCQF5E8oD89UXoHn9JFEhJUz"),
            pk("SysvarRent111111111111111111111111111111111"),
            pk("Hq2wp8uJ9jCPsYgNHex8RtqdvMPfVGoYwjvF1ATiwn2Y"),
            pk("pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ"), // FEE_PROGRAM (placeholder)
        ),
    }
}
