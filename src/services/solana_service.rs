use anyhow::{Context, anyhow, Result};
use bincode;
use borsh::BorshSerialize;
use borsh::BorshDeserialize;
use bs58;
use sha2::{Digest, Sha256};
use solana_client::nonblocking::rpc_client::RpcClient as AsyncRpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    hash::Hash,
    compute_budget::ComputeBudgetInstruction,
    config::program,
    instruction::{AccountMeta, Instruction},
    message::Message, pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer},
    system_program, transaction::Transaction
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use std::{time::Duration, path::Path, str::FromStr};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signature::Signature;
use solana_transaction_status::UiTransactionEncoding;

use crate::api::premarket;
use crate::models::premarket::{PremarketState, PremarketListResult, TokenDynamicInfo, HolderInfo, LinkType, CommunityLink, CommunityInfoServiceModel, FullPremarketInfo, PremarketInfoServiceModel, TokenInfo, TokenLinks, PremarketGoal, UserInfoShort, JoinConfirmationStatusDTO, OutConfirmationStatusDTO, PremarketOnchainUser, PremarketOnchainData};

use crate::models::premarket::{
    BuildFinishTxParams, 
    BuildKillTxParams, 
    BuildPremarketTxParams,
    BuiltTx, 
    DistributeTokensParams, 
    GetPremarketDataParams, 
    SolanaNetwork, 
    UpdatePremarketDataParams, 
    DeployTxParams,
    CheckTxParams,
};

use crate::api::premarket::CheckTxResponse;

use crate::storage::signing_keys::{
    get_mint_signing_keypair_by_premarket,
    insert_mint_signing_key,
    delete_signing_key_by_pubkey
};
use serde_json;
use sqlx::PgPool;

use spl_associated_token_account::ID as associated_token_program_id;
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token::ID as token_program_id;
use tokio::time::sleep;

use crate::storage::signing_keys::get_unused_signing_key;


impl TryFrom<&str> for SolanaNetwork {
    type Error = anyhow::Error;
    fn try_from(s: &str) -> Result<Self> {
        match s {
            "devnet" => Ok(Self::Devnet),
            "mainnet-beta" | "mainnet_beta" | "mainnetbeta" => Ok(Self::MainnetBeta),
            _ => Err(anyhow!("unsupported network: {}", s)),
        }
    }
}

const CREATE_METHOD_NAME: &str = "create_premarket";
const JOIN_METHOD_NAME: &str = "join_to_premarket";
const OUT_METHOD_NAME: &str  = "out_of_premarket";
const FINISH_METHOD_NAME: &str = "finish_premarket";
const KILL_METHOD_NAME: &str = "kill_premarket";
const DISTRIBUTE_METHOD_NAME: &str = "distribute_tokens";

fn pk(s: &str) -> Pubkey { Pubkey::from_str(s).expect("invalid pubkey") }

fn rpc_url(network: SolanaNetwork) -> String {
    match network {
        SolanaNetwork::Devnet =>
            std::env::var("SOLANA_DEVNET_RPC").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string()),
        SolanaNetwork::MainnetBeta =>
            std::env::var("SOLANA_MAINNET_RPC").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
    }
}

fn program_id_for(network: SolanaNetwork) -> Pubkey {
    let (env_key, fallback) = match network {
        SolanaNetwork::Devnet      => ("PURPLE_PROGRAM_ID_DEV",  "ERCTELKB8tWDcLw4hLLYmxk9kirci5tP3BG4NntpwoAj"),
        SolanaNetwork::MainnetBeta => ("PURPLE_PROGRAM_ID_MAIN", "ERCTELKB8tWDcLw4hLLYmxk9kirci5tP3BG4NntpwoAj"),
    };

    match std::env::var(env_key) {
        Ok(v) => Pubkey::from_str(&v)
            .unwrap_or_else(|_| panic!("invalid {} pubkey: {}", env_key, v)),
        Err(_) => {
            eprintln!("WARN: {} not set; using fallback {}", env_key, fallback);
            Pubkey::from_str(fallback).expect("invalid fallback program id")
        }
    }
}


fn read_revelcy_auth(network: SolanaNetwork) -> Keypair {
    let var = match network {
        SolanaNetwork::Devnet => "REVELCY_AUTH_PRIVATE_KEY_DEV",
        SolanaNetwork::MainnetBeta => "REVELCY_AUTH_PRIVATE_KEY_MAIN",
    };

    let raw = std::env::var(var).unwrap_or_else(|_| panic!("env {var} required"));
    let s = raw.trim();

    if s.ends_with(".json") || Path::new(s).exists() {
        return read_keypair_file(s).expect("failed to read keypair file");
    }

    if s.starts_with('[') {
        let bytes: Vec<u8> = serde_json::from_str(s).expect("invalid json keypair bytes");
        assert_len_64(&bytes, "json bytes");
        return Keypair::from_bytes(&bytes).expect("invalid keypair bytes (json)");
    }

    if let Some(b64) = s.strip_prefix("base64:") {
        let bytes = BASE64.decode(b64).expect("invalid base64 key");
        assert_len_64(&bytes, "base64 bytes");
        return Keypair::from_bytes(&bytes).expect("invalid keypair bytes (base64)");
    }
    if s.contains('=') || s.contains('/') || s.contains('+') {
        if let Ok(bytes) = BASE64.decode(s) {
            assert_len_64(&bytes, "base64 bytes");
            return Keypair::from_bytes(&bytes).expect("invalid keypair bytes (base64)");
        }
    }

    let bytes = bs58::decode(s).into_vec().expect("invalid base58 key");
    assert_len_64(&bytes, "base58 bytes");
    Keypair::from_bytes(&bytes).expect("invalid keypair bytes (base58)")
}

fn pda(program: &Pubkey, seeds: &[&[u8]]) -> (Pubkey, u8) {
    Pubkey::find_program_address(seeds, program)
}

fn constants(network: SolanaNetwork) -> (
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


#[inline]
fn assert_len_64(bytes: &[u8], label: &str) {
    if bytes.len() != 64 {
        panic!("expected 64 bytes for {}, got {}", label, bytes.len());
    }
}

fn anchor_sighash_global(name: &str) -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(b"global:");
    hasher.update(name.as_bytes());
    let hash = hasher.finalize();
    let mut out = [0u8; 8];
    out.copy_from_slice(&hash[..8]);
    out
}


#[derive(BorshSerialize)]
struct CreatePremarketArgsBorsh {
    end_timestamp: i64,
    goal_sol: u64,
    max_sol: u64,
    name: String,
    symbol: String,
    uri: String,
    amount_in_lamports: u64,
}

pub async fn build_create_premarket_tx(
    pool: &PgPool,
    params: BuildPremarketTxParams,
) -> Result<BuiltTx> {
    let program_id = program_id_for(params.network);
    let rpc = AsyncRpcClient::new_with_timeout(rpc_url(params.network), Duration::from_secs(15));

    let mint = if let Some(pair) = get_unused_signing_key(pool).await? {
        let bytes = parse_privkey_64(&pair.priv_key)
            .context("signing_keys.priv_key parse failed")?;
        Keypair::from_bytes(&bytes).context("invalid keypair bytes in signing_keys")?
    } else {
        Keypair::new()
    };


    let mint_pub = mint.pubkey().to_string();

    println!("Using mint pubkey: {}", mint_pub);

    delete_signing_key_by_pubkey(pool, &mint_pub).await?;

    let revelcy = read_revelcy_auth(params.network);
    let revelcy_pub = revelcy.pubkey();
    let (premarket_pda, _bump) =
        Pubkey::find_program_address(&[revelcy_pub.as_ref(), mint.pubkey().as_ref()], &program_id);

    let priv_b58 = bs58::encode(mint.to_bytes()).into_string();
    insert_mint_signing_key(pool, &premarket_pda.to_string(), &mint_pub, &priv_b58)
        .await
        .context("failed to insert mint key into signing_keys")?;


    // всё дальнейшее — в одном блоке, чтобы при Err сделать cleanup
    let result: Result<BuiltTx> = async {
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
            AccountMeta::new_readonly(revelcy_pub, true), // revelcy_auth (signer)
            AccountMeta::new(premarket_pda, false),       // premarket_account (writable)
            AccountMeta::new_readonly(mint.pubkey(), false), // mint
            AccountMeta::new(params.user, true),          // user (writable, signer)
            AccountMeta::new_readonly(system_program::ID, false), // system_program
        ];
        let ix = Instruction { program_id, accounts, data };

        let blockhash = get_valid_latest_blockhash(&rpc, 50).await.context("get_latest_blockhash failed")?;

        let msg = Message::new(&[ix], Some(&params.user));
        let mut tx = Transaction::new_unsigned(msg);
        tx.try_partial_sign(&[&revelcy], blockhash)
            .context("failed to partially sign transaction with revelcyAuth")?;

        let raw = bincode::serialize(&tx).context("bincode serialize(Transaction) failed")?;
        let tx_b64 = BASE64.encode(raw);

        Ok(BuiltTx { tx_base64: tx_b64, premarket_pda })
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

#[derive(borsh::BorshSerialize)]
struct JoinArgsBorsh {
    amount_in_lamports: u64,
}

// join → async + nonblocking
pub async fn build_join_premarket_tx(
    params: crate::models::premarket::BuildJoinTxParams,
) -> Result<crate::models::premarket::BuiltTx> {
    let program_id = program_id_for(params.network);
    let rpc = AsyncRpcClient::new_with_timeout(rpc_url(params.network), Duration::from_secs(15));

    // data = discriminator + borsh(args)
    let mut data = Vec::with_capacity(8 + 16);
    data.extend_from_slice(&anchor_sighash_global(JOIN_METHOD_NAME));
    JoinArgsBorsh { amount_in_lamports: params.amount }
        .serialize(&mut data)
        .map_err(|e| anyhow!("borsh serialize failed: {e}"))?;

    let revelcy = read_revelcy_auth(params.network);
    let revelcy_pub = revelcy.pubkey();

    // IDL:
    // 1) revelcy_auth (writable, signer)
    // 2) user         (writable, signer)
    // 3) premarket_account (writable)
    // 4) system_program
    let accounts = vec![
        AccountMeta::new(revelcy_pub, true),
        AccountMeta::new(params.user, true),
        AccountMeta::new(params.premarket, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];

    let ix = Instruction { program_id, accounts, data };
    let blockhash = get_valid_latest_blockhash(&rpc, 50).await.context("get_latest_blockhash failed")?;
    let msg = Message::new(&[ix], Some(&params.user));
    let mut tx = Transaction::new_unsigned(msg);

    tx.try_partial_sign(&[&revelcy], blockhash)?;

    let raw = bincode::serialize(&tx)?;
    let tx_b64 = BASE64.encode(raw);
    Ok(crate::models::premarket::BuiltTx { tx_base64: tx_b64, premarket_pda: params.premarket })
}

// out → async + nonblocking
pub async fn build_out_premarket_tx(
    params: crate::models::premarket::BuildOutTxParams,
) -> Result<crate::models::premarket::BuiltTx> {
    let program_id = program_id_for(params.network);
    let rpc = AsyncRpcClient::new_with_timeout(rpc_url(params.network), Duration::from_secs(15));

    let mut data = Vec::with_capacity(8);
    data.extend_from_slice(&anchor_sighash_global(OUT_METHOD_NAME));

    let revelcy = read_revelcy_auth(params.network);
    let revelcy_pub = revelcy.pubkey();

    // IDL:
    // 1) revelcy_auth (signer)
    // 2) user         (writable, signer)
    // 3) premarket_account (writable)
    // 4) system_program
    let accounts = vec![
        AccountMeta::new_readonly(revelcy_pub, true),
        AccountMeta::new(params.user, true),
        AccountMeta::new(params.premarket, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];

    let ix = Instruction { program_id, accounts, data };
    let blockhash = get_valid_latest_blockhash(&rpc, 50).await.context("get_latest_blockhash failed")?;
    let msg = Message::new(&[ix], Some(&params.user));
    let mut tx = Transaction::new_unsigned(msg);
    tx.try_partial_sign(&[&revelcy], blockhash)?;

    let raw = bincode::serialize(&tx)?;
    let tx_b64 = BASE64.encode(raw);
    Ok(crate::models::premarket::BuiltTx { tx_base64: tx_b64, premarket_pda: params.premarket })
}

pub async fn build_finish_premarket_tx(
    pool: &PgPool,
    params: BuildFinishTxParams,
) -> Result<BuiltTx> {
    let program_id = program_id_for(params.network);
    let rpc = AsyncRpcClient::new_with_timeout(rpc_url(params.network), Duration::from_secs(15));
    let (mint_auth, pump_fun_program_id, pumpfun_global, metaplex_program, event_auth,
         fee_recipient, rent_sysvar, global_volume_accum, fee_program) = constants(params.network);

    // mint key из БД
    let pair = get_mint_signing_keypair_by_premarket(pool, &params.premarket.to_string())
        .await
        .context("signing_keys: mint key not found for this premarket")?
        .ok_or_else(|| anyhow!("mint key not found for premarket {}", params.premarket))?;

    let mint_bytes = parse_privkey_64(&pair.priv_key).context("mint priv_key parse failed")?;
    if mint_bytes.len() != 64 {
        return Err(anyhow!("mint priv_key must be 64 bytes, got {}", mint_bytes.len()));
    }
    let mint_kp = Keypair::from_bytes(&mint_bytes).context("mint priv_key: invalid keypair bytes")?;
    let mint_pub = mint_kp.pubkey();

    // PDAs/ATAs
    let (bonding_curve, _) = pda(&pump_fun_program_id, &[b"bonding-curve", mint_pub.as_ref()]);
    let bonding_curve_ata = get_associated_token_address(&bonding_curve, &mint_pub);
    let (metadata, _) = pda(&metaplex_program, &[b"metadata", metaplex_program.as_ref(), mint_pub.as_ref()]);
    let revelcy = read_revelcy_auth(params.network);
    let revelcy_pub = revelcy.pubkey();
    let associated_user_ata = get_associated_token_address(&revelcy_pub, &mint_pub);
    let (creator_vault, _) = pda(&pump_fun_program_id, &[b"creator-vault", params.user.as_ref()]);
    let (user_volume_accum, _) = pda(&pump_fun_program_id, &[b"user_volume_accumulator", revelcy_pub.as_ref()]);
    let seed1: &[u8] = b"fee_config";
    let seed2: [u8; 32] = [
        1, 86, 224, 246, 147, 102, 90, 207,
        68, 219, 21, 104, 191, 23, 91, 170,
        81, 137, 203, 151, 245, 210, 255, 59,
        101, 93, 43, 182, 253, 109, 24, 176,
    ];
    let (fee_config, _) = Pubkey::find_program_address(&[seed1, &seed2], &fee_program);

    // только discriminator
    let mut data = Vec::with_capacity(8);
    data.extend_from_slice(&anchor_sighash_global(FINISH_METHOD_NAME));

    // IDL порядок (writable/signer строго как описано):
    let accounts = vec![
        AccountMeta::new(revelcy_pub, true),                // 1) revelcy_auth (writable, signer)
        AccountMeta::new(params.premarket, false),          // 2) premarket_account (writable)
        AccountMeta::new(mint_pub, true),                   // 3) token_mint (writable, signer)
        AccountMeta::new_readonly(mint_auth, false),        // 4) mint_auth
        AccountMeta::new(bonding_curve, false),             // 5) bonding_curve (writable)
        AccountMeta::new(bonding_curve_ata, false),         // 6) bonding_curve_ata (writable)
        AccountMeta::new(pumpfun_global, false),            // 7) global (writable)
        AccountMeta::new(metaplex_program, false),          // 8) mpl_token_metadata (writable)
        AccountMeta::new(metadata, false),                  // 9) metadata (writable)
        AccountMeta::new(params.user, true),                // 10) user (writable, signer)
        AccountMeta::new_readonly(system_program::ID, false),          // 11) system_program
        AccountMeta::new_readonly(token_program_id, false),            // 12) token_program
        AccountMeta::new_readonly(associated_token_program_id, false), // 13) associated_token_program
        AccountMeta::new_readonly(rent_sysvar, false),                 // 14) rent
        AccountMeta::new(event_auth, false),                // 15) event_auth (writable)
        AccountMeta::new_readonly(pump_fun_program_id, false), // 16) pump_fun_program_id
        AccountMeta::new(fee_recipient, false),             // 17) fee_recipient (writable)
        AccountMeta::new(associated_user_ata, false),       // 18) associated_user (writable)
        AccountMeta::new(creator_vault, false),             // 19) creator_vault (writable)
        AccountMeta::new(global_volume_accum, false),       // 20) global_volume_accumulator (writable)
        AccountMeta::new(user_volume_accum, false),         // 21) user_volume_accumulator (writable)
        AccountMeta::new(fee_config, false),                // 22) fee_config (writable)
        AccountMeta::new(fee_program, false),               // 23) fee_program (writable)
    ];

    let ix_finish = Instruction { program_id, accounts, data };
    let ix_compute = ComputeBudgetInstruction::set_compute_unit_limit(400_000);

    let blockhash = get_valid_latest_blockhash(&rpc, 50).await.context("get_latest_blockhash failed")?;
    let msg = Message::new(&[ix_compute, ix_finish], Some(&params.user));
    let mut tx = Transaction::new_unsigned(msg);

    tx.try_partial_sign(&[&revelcy, &mint_kp], blockhash)
        .context("partial sign (revelcy + mint) failed")?;

    let raw = bincode::serialize(&tx).context("serialize tx failed")?;
    let tx_b64 = BASE64.encode(raw);

    Ok(BuiltTx { tx_base64: tx_b64, premarket_pda: params.premarket })
}

pub async fn distribute_tk(
    pool: &PgPool,
    params: DistributeTokensParams,
) -> Result<u64> {
    //extracting params 
    //preparing tx
    //sending tx 
    //geting result 
    //returning result 

    let program_id = program_id_for(params.network);
    let client = AsyncRpcClient::new_with_timeout(rpc_url(params.network), Duration::from_secs(15));
    let revelcy_auth = read_revelcy_auth(params.network);
    let premarket_account = params.premarket;
    let system_program = system_program::ID;
    let all_entered_users = params.users;
    let token_mint = params.token_mint;

    println!("Premarket Account: {}", premarket_account);
    println!("Token Mint: {}", token_mint);
    println!("All Entered Users: {:?}", all_entered_users);

    let revelcy_auth_ata = get_associated_token_address(&revelcy_auth.pubkey(), &token_mint);
    println!("Revelcy Auth: {}", revelcy_auth.pubkey());
    let mut accounts = vec![
        AccountMeta::new(revelcy_auth.pubkey(), true),
        AccountMeta::new(revelcy_auth_ata, false),
        AccountMeta::new(premarket_account, false),
        AccountMeta::new(token_mint, false),
        AccountMeta::new_readonly(system_program, false),
        AccountMeta::new_readonly(token_program_id, false),
        AccountMeta::new_readonly(associated_token_program_id, false),
    ];

    for user in all_entered_users {
        let user_ata = get_associated_token_address(&Pubkey::from_str(&user).unwrap(), &token_mint);
        accounts.push(AccountMeta::new(user_ata, false));
        accounts.push(AccountMeta::new(Pubkey::from_str(&user).unwrap(), false));
        println!("User account: {}", user);
        println!("User ATA: {}", user_ata);
    }

    let discriminator: [u8; 8] = [
        105,
        69,
        130,
        52,
        196,
        28,
        176,
        120
    ];

    let mut data = Vec::with_capacity(8);
    //data.extend_from_slice(&discriminator);
    data.extend_from_slice(&anchor_sighash_global(DISTRIBUTE_METHOD_NAME));


    let ix = Instruction { program_id, accounts, data };
    let add_cu_ix = ComputeBudgetInstruction::set_compute_unit_limit(1_000_000);

    let blockhash = get_valid_latest_blockhash(&client, 50).await.context("get_latest_blockhash failed")?;


    let tx = Transaction::new_signed_with_payer(
        &[add_cu_ix, ix],
        Some(&revelcy_auth.pubkey()),
        &[&revelcy_auth],
        blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx).await.context("send_and_confirm_transaction failed")?;
    println!("tx signature: {}", sig);

    Ok(1)

}

pub async fn build_kill_premarket_tx(
    pool: &PgPool,
    params: BuildKillTxParams,
) -> Result<BuiltTx> {
    //extract params 
    //build tx
    //return tx build 

    let program_id = program_id_for(params.network);
    let client = AsyncRpcClient::new_with_timeout(rpc_url(params.network), Duration::from_secs(15));
    let revelcy = read_revelcy_auth(params.network);
    let revelcy_pub = revelcy.pubkey();
    let premarket_account = params.premarket;
    let system_program = system_program::ID;
    let all_entered_users = params.users;

    let mut accounts = vec![
        AccountMeta::new(revelcy_pub, true),
        AccountMeta::new(premarket_account, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    for user in all_entered_users {
        accounts.push(AccountMeta::new(Pubkey::from_str(&user).unwrap(), false));
    }

    let discriminator: [u8; 8] = [
        10,
        112,
        216,
        238,
        253,
        26,
        122,
        160
    ];

    let mut data = Vec::with_capacity(8);
    //data.extend_from_slice(&discriminator);
    data.extend_from_slice(&anchor_sighash_global(KILL_METHOD_NAME));


    let ix = Instruction { program_id, accounts, data };
    let blockhash = get_valid_latest_blockhash(&client, 50).await.context("get_latest_blockhash failed")?;
    let msg = Message::new(&[ix], Some(&params.user));
    let mut tx = Transaction::new_unsigned(msg);
    tx.try_partial_sign(&[&revelcy], blockhash)
        .context("partial sign (revelcy + mint) failed")?;

    let raw = bincode::serialize(&tx).context("serialize tx failed")?;
    let tx_b64 = BASE64.encode(raw);

    Ok(BuiltTx { tx_base64: tx_b64, premarket_pda: params.premarket })
}

pub async fn test_build_kill_premarket_tx(
    pool: &PgPool,
    params: BuildKillTxParams,
) -> Result<()> {
    //extract params 
    //build tx
    //return tx build 

    let program_id = program_id_for(params.network);
    let client = AsyncRpcClient::new_with_timeout(rpc_url(params.network), Duration::from_secs(15));
    let revelcy = read_revelcy_auth(params.network);
    let revelcy_pub = revelcy.pubkey();
    let premarket_account = params.premarket;
    let system_program = system_program::ID;
    let all_entered_users = params.users;

    let mut accounts = vec![
        AccountMeta::new(revelcy_pub, true),
        AccountMeta::new(premarket_account, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    for user in all_entered_users {
        accounts.push(AccountMeta::new(Pubkey::from_str(&user).unwrap(), false));
    }

    let discriminator: [u8; 8] = [
        10,
        112,
        216,
        238,
        253,
        26,
        122,
        160
    ];

    let mut data = Vec::with_capacity(8);
    //data.extend_from_slice(&discriminator);
    data.extend_from_slice(&anchor_sighash_global(KILL_METHOD_NAME));


    let ix = Instruction { program_id, accounts, data };
    let blockhash = get_valid_latest_blockhash(&client, 50).await.context("get_latest_blockhash failed")?;

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&revelcy.pubkey()),
        &[&revelcy],
        blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx).await?;
    println!("tx signature: {}", sig);

    Ok(())
}

pub async fn get_premarket_data(
    params: GetPremarketDataParams,
) -> Result<PremarketOnchainData, Box<dyn std::error::Error>> {

    let client = AsyncRpcClient::new_with_timeout(rpc_url(params.network), Duration::from_secs(15));

    let account = client.get_account(&params.premarket).await?;
    
    println!("Account owner: {}", account.owner);
    println!("Account data length: {} bytes", account.data.len());
    
    // Skip the 8-byte discriminator
    let data = &account.data[8..];

    // Get the user vector length
    let users_len = u32::from_le_bytes(data[0..4].try_into()?);
    println!("\nUsers vector length: {}", users_len);
    
    // Calculate offset after the user vector
    // Format: 4 bytes for length + (users_len * (32 bytes for pubkey + 8 bytes for lamports))
    let offset_after_users = 4 + (users_len as usize * 40);
    let mut all_entered_users = Vec::<PremarketOnchainUser>::new();

    // Extract user data if available
    if users_len > 0 {
        for i in 0..users_len as usize {
            let user_offset = 4 + (i * 40); // 4 bytes for vector length + i * entry size
            let pubkey_bytes = &data[user_offset..user_offset+32];
            let pubkey = Pubkey::new_from_array(pubkey_bytes.try_into()?);

            let lamports_bytes = &data[user_offset+32..user_offset+40];
            let lamports = u64::from_le_bytes(lamports_bytes.try_into()?);

            let user = PremarketOnchainUser {
                wallet: pubkey,
                contributed_lamports: lamports,
            };
            all_entered_users.push(user);

            let lamports_bytes = &data[user_offset+32..user_offset+40];
            let lamports = u64::from_le_bytes(lamports_bytes.try_into()?);
            println!("User {}: {} contributed {} lamports", i, pubkey, lamports);
        }
    }
    
    // Now extract the rest of the fields after the user vector
    let end_timestamp = i64::from_le_bytes(data[offset_after_users..offset_after_users+8].try_into()?);
    println!("End timestamp: {}", end_timestamp);
    
    let end_timestamp_updated = data[offset_after_users+8] != 0;
    println!("End timestamp updated: {}", end_timestamp_updated);
    
    let goal_sol = u64::from_le_bytes(data[offset_after_users+9..offset_after_users+17].try_into()?);
    println!("Goal SOL: {}", goal_sol);
    
    let max_sol = u64::from_le_bytes(data[offset_after_users+17..offset_after_users+25].try_into()?);
    println!("Max SOL: {}", max_sol);
    
    let mint = Pubkey::new_from_array(data[offset_after_users+25..offset_after_users+57].try_into()?);
    println!("Mint: {}", mint);

    Ok(PremarketOnchainData {
        users: all_entered_users,
        end_timestamp,
        goal_lamports: goal_sol,
        max_lamports: max_sol,
        mint,
    })
}

pub async fn update_premarket_data(
    pool: &PgPool,
    params: UpdatePremarketDataParams,
) -> Result<BuiltTx> {
    let network = SolanaNetwork::try_from(params.network.as_str())
        .map_err(|_| actix_web::error::ErrorBadRequest("invalid network")).unwrap();

    let program_id = program_id_for(network);

    let client = AsyncRpcClient::new_with_timeout(rpc_url(network), Duration::from_secs(15));

    let revelcy_auth = read_revelcy_auth(network);

    let premarket_account = Pubkey::from_str(&params.premarket_account)?;

    let system_program = system_program::ID;

    let accounts = vec![
        AccountMeta::new(revelcy_auth.pubkey(), true),
        AccountMeta::new(Pubkey::from_str(&params.user_pubkey)?, true),
        AccountMeta::new(premarket_account, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    #[derive(BorshDeserialize, BorshSerialize)]
    pub struct UpdatePremarketDataArgs {
        pub end_timestamp: Option<i64>,
        pub end_timestamp_updated: Option<bool>,
        pub goal_sol: Option<u64>,
        pub max_sol: Option<u64>,
        pub mint: Option<String>,
        pub name: Option<String>,
        pub symbol: Option<String>,
        pub uri: Option<String>,
        pub creator: Option<String>,
    }


    let args = UpdatePremarketDataArgs {
        end_timestamp: params.end_timestamp,
        end_timestamp_updated: params.end_timestamp_updated,
        goal_sol: params.goal_sol,
        max_sol: params.max_sol,
        mint: params.mint,
        name: params.name,
        symbol: params.symbol,
        uri: params.uri,
        creator: params.creator,
    };

    let discriminator: [u8; 8] = [
        20,
        82,
        102,
        101,
        150,
        216,
        162,
        52
    ];

    let mut data = Vec::with_capacity(8 + args.try_to_vec().unwrap().len());
    data.extend_from_slice(&discriminator);
    data.extend(args.try_to_vec().unwrap());

    let ix = Instruction { program_id, accounts, data };
    
    let blockhash = get_valid_latest_blockhash(&client, 50).await.context("get_latest_blockhash failed")?;
    let msg = Message::new(&[ix], Some(&Pubkey::from_str(&params.user_pubkey)?));
    let mut tx = Transaction::new_unsigned(msg);
    tx.try_partial_sign(&[&revelcy_auth], blockhash)
        .context("partial sign (revelcy + mint) failed")?;

    let raw = bincode::serialize(&tx).context("serialize tx failed")?;
    let tx_b64 = BASE64.encode(raw);

    Ok(BuiltTx { tx_base64: tx_b64, premarket_pda: premarket_account })
}

// move me to utils
fn parse_privkey_64(s: &str) -> Result<Vec<u8>> {
    let s = s.trim();

    // JSON-массив: "[1,2,3,...,64]"
    if s.starts_with('[') && s.ends_with(']') {
        let v: Vec<u8> = serde_json::from_str(s).context("invalid JSON priv_key")?;
        anyhow::ensure!(v.len() == 64, "json priv_key must be 64 bytes, got {}", v.len());
        return Ok(v);
    }

    // CSV: "1,2,3,...,64"
    if s.contains(',') && !s.contains(':') && !s.contains('[') && !s.contains(']') {
        let v: Result<Vec<u8>, _> = s.split(',')
            .map(|x| x.trim().parse::<u8>())
            .collect();
        let v = v.context("invalid CSV priv_key (non-numeric token)")?;
        anyhow::ensure!(v.len() == 64, "csv priv_key must be 64 bytes, got {}", v.len());
        return Ok(v);
    }

    // base64 c префиксом
    if let Some(b64) = s.strip_prefix("base64:") {
        let v = BASE64.decode(b64).context("invalid base64 priv_key")?;
        anyhow::ensure!(v.len() == 64, "base64 priv_key must be 64 bytes, got {}", v.len());
        return Ok(v);
    }

    // «сырой» base64 (эвристика)
    if s.contains('=') || s.contains('/') || s.contains('+') {
        if let Ok(v) = BASE64.decode(s) {
            anyhow::ensure!(v.len() == 64, "base64 priv_key must be 64 bytes, got {}", v.len());
            return Ok(v);
        }
    }

    // fallback: base58
    let v = bs58::decode(s).into_vec().context("invalid base58 priv_key")?;
    anyhow::ensure!(v.len() == 64, "base58 priv_key must be 64 bytes, got {}", v.len());
    Ok(v)
}

// !!!now user is CONST i need to change it later!!!
pub async fn deploy_tx_service(
    pool: &PgPool,
    params: DeployTxParams,
) -> Result<()> {
    let network = SolanaNetwork::try_from(params.network.as_str())
        .map_err(|_| actix_web::error::ErrorBadRequest("invalid network")).unwrap();
    let client = AsyncRpcClient::new_with_timeout(rpc_url(network), Duration::from_secs(15));

    // Deserialize the base64 transaction
    let tx_bytes = BASE64.decode(&params.tx)
        .context("failed to decode base64 transaction")?;
    let mut tx: Transaction = bincode::deserialize(&tx_bytes)
        .context("failed to deserialize transaction")?;

    // Get the user's keypair (you'll need to pass this in params or get it somehow)
    let user_keypair = Keypair::from_base58_string("45PRZt1VKD4rj98hwAko5G2M2ngyd2NtDVJ1Hf1f1MdtL5DxZDFNXopaDZ55r6EtH6MmDixe3DHLu94XUBvtjohT");

    // Sign the transaction with the user keypair
    // Note: The transaction should already be partially signed by revelcy_auth
    tx.try_sign(&[&user_keypair], tx.message.recent_blockhash)
        .context("failed to sign transaction with user keypair")?;

    // Send and confirm the transaction
    let sig = client.send_and_confirm_transaction(&tx).await
        .context("failed to send and confirm transaction")?;
    
    println!("Transaction signature: {}", sig);

    Ok(())
}

pub async fn get_valid_latest_blockhash(
    client: &AsyncRpcClient,
    max_retries: usize,
) -> Result<Hash> {
    let mut retries = 0;
    let mut recent_blockhash = client
        .get_latest_blockhash()
        .await
        .context("Failed to get blockhash")?;

    loop {
        match client
            .is_blockhash_valid(&recent_blockhash, CommitmentConfig::processed())
            .await
        {
            Ok(true) => return Ok(recent_blockhash),
            Ok(false) | Err(_) => {
                retries += 1;
                if retries >= max_retries {
                    return Err(anyhow!("Blockhash still invalid after {} retries", max_retries));
                }

                recent_blockhash = client
                    .get_latest_blockhash()
                    .await
                    .context("Failed to get blockhash")?;

                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        }
    }
}

pub async fn check_tx_service(
    pool: &PgPool,
    params: CheckTxParams,
) -> Result<CheckTxResponse, actix_web::Error> {
    let network = SolanaNetwork::try_from(params.network.as_str())
        .map_err(|_| actix_web::error::ErrorBadRequest("invalid network")).unwrap();
    let client = AsyncRpcClient::new_with_timeout(rpc_url(network), Duration::from_secs(15));

    let tx_signature = match solana_sdk::signature::Signature::from_str(&params.sig) {
        Ok(sig) => sig,
        Err(_) => return Err(actix_web::error::ErrorBadRequest("Invalid transaction signature format")),
    };

    let status = client
        .get_signature_status(&tx_signature)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to get transaction status: {}", e)))?;

    println!("Transaction status: {:?}", status);

    match status {
        Some(status) if status.is_ok() => {
            let tx_result = client
                .get_transaction(&tx_signature, UiTransactionEncoding::JsonParsed)
                .await;
            match tx_result {
                Ok(tx) => {
                    if let Some(meta) = &tx.transaction.meta {
                        let encoded_transaction = &tx.transaction.transaction;
                        match encoded_transaction {
                            solana_transaction_status::EncodedTransaction::Json(ui_transaction) => {
                                let message = &ui_transaction.message;
                                match message {
                                    solana_transaction_status::UiMessage::Raw(_) => {
                                    },
                                    solana_transaction_status::UiMessage::Parsed(parsed_message) => {
                                        for instruction in &parsed_message.instructions {
                                            match instruction {
                                                solana_transaction_status::UiInstruction::Parsed(parsed_ix) => {
                                                    match parsed_ix {
                                                        solana_transaction_status::UiParsedInstruction::Parsed(_) => {
                                                        },
                                                        solana_transaction_status::UiParsedInstruction::PartiallyDecoded(partially_decoded) => {
                                                            if partially_decoded.program_id == program_id_for(network).to_string() {
                                                                if partially_decoded.data.starts_with("2cDhewTfU1T") {
                                                                    println!("Create instruction detected");
                                                                    let premarket_pda = &partially_decoded.accounts[1];
                                                                    println!("Premarket PDA: {}", premarket_pda);
                                                                    let premarket_data = get_premarket_data(GetPremarketDataParams {
                                                                        network: network,
                                                                        premarket: Pubkey::from_str(premarket_pda).map_err(|_| actix_web::error::ErrorBadRequest("Invalid premarket PDA format"))?,
                                                                    }).await.map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to get premarket data: {}", e)))?;
                                                                    println!("Premarket Data: {:?}", premarket_data);
                                                                    let output = CheckTxResponse{
                                                                        user_pubkey: None,
                                                                        name: None,
                                                                        symbol: None,
                                                                        uri: None,
                                                                        deadline: Some(premarket_data.end_timestamp),
                                                                        goal_sol_lamp: Some(premarket_data.goal_lamports),
                                                                        max_sol_lamp: Some(premarket_data.max_lamports),
                                                                        creator_allocate_lamp: None,
                                                                        premarket: Some(premarket_pda.clone()),
                                                                        lamports_in: None,
                                                                    };
                                                                    return Ok(output);
                                                                }
                                                                if partially_decoded.data.starts_with("YVJ16Tm7Yjx") {
                                                                    println!("Join instruction detected");
                                                                    let user = &partially_decoded.accounts[1];
                                                                    println!("Joined User: {}", user);
                                                                    let premarket_pda = &partially_decoded.accounts[2];
                                                                    println!("Premarket PDA: {}", premarket_pda);
                                                                    let premarket_data = get_premarket_data(GetPremarketDataParams {
                                                                        network: network,
                                                                        premarket: Pubkey::from_str(premarket_pda).map_err(|_| actix_web::error::ErrorBadRequest("Invalid premarket PDA format"))?,
                                                                    }).await.map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to get premarket data: {}", e)))?;
                                                                    let user_pubkey = Pubkey::from_str(user).map_err(|_| actix_web::error::ErrorBadRequest("Invalid user pubkey format"))?;
                                                                    let user_found = premarket_data.users.iter().find(|u| u.wallet == user_pubkey);
                                                                    match user_found {
                                                                        Some(premarket_user) => {
                                                                            println!("User found in premarket! Wallet: {}, Contributed: {} lamports", 
                                                                                premarket_user.wallet, premarket_user.contributed_lamports);
                                                                            let output = CheckTxResponse{
                                                                                user_pubkey: Some(premarket_user.wallet.to_string()),
                                                                                name: None,
                                                                                symbol: None,
                                                                                uri: None,
                                                                                deadline: None,
                                                                                goal_sol_lamp: None,
                                                                                max_sol_lamp: None,
                                                                                creator_allocate_lamp: None,
                                                                                premarket: None,
                                                                                lamports_in: Some(premarket_user.contributed_lamports),
                                                                            };
                                                                            return Ok(output);
                                                                        },
                                                                        None => { 
                                                                            println!("User {} not found in premarket data", user);
                                                                            // Additional logic for when user is not found
                                                                        } 
                                                                    }
                                                                }
                                                                if partially_decoded.data.starts_with("Ej2HdfD46xq") {
                                                                    println!("Out instruction detected");
                                                                    let user = &partially_decoded.accounts[1];
                                                                    println!("User: {}", user);
                                                                    let premarket_pda = &partially_decoded.accounts[2];
                                                                    println!("Premarket PDA: {}", premarket_pda);
                                                                    let premarket_data = get_premarket_data(GetPremarketDataParams {
                                                                        network: network,
                                                                        premarket: Pubkey::from_str(premarket_pda).map_err(|_| actix_web::error::ErrorBadRequest("Invalid premarket PDA format"))?,
                                                                    }).await.map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to get premarket data: {}", e)))?;
                                                                    let user_pubkey = Pubkey::from_str(user).map_err(|_| actix_web::error::ErrorBadRequest("Invalid user pubkey format"))?;
                                                                    let user_found = premarket_data.users.iter().find(|u| u.wallet == user_pubkey);
                                                                    match user_found {
                                                                        Some(premarket_user) => {
                                                                            println!("Error! User found in premarket! Wallet: {}, Contributed: {} lamports", 
                                                                                premarket_user.wallet, premarket_user.contributed_lamports);
                                                                            // Additional logic for when user is found
                                                                        },
                                                                        None => {
                                                                            println!("User {} not found in premarket data", user);
                                                                            let output = CheckTxResponse{
                                                                                user_pubkey: Some(user_pubkey.to_string()),
                                                                                name: None,
                                                                                symbol: None,
                                                                                uri: None,
                                                                                deadline: None,
                                                                                goal_sol_lamp: None,
                                                                                max_sol_lamp: None,
                                                                                creator_allocate_lamp: None,
                                                                                premarket: None,
                                                                                lamports_in: None,
                                                                            };
                                                                            return Ok(output);
                                                                        }
                                                                    }
                                                                }
                                                                if partially_decoded.data.starts_with("NbHPPfXBWVg") {
                                                                    println!("Finish instruction detected");
                                                                    let bonding_curve = &partially_decoded.accounts[4];
                                                                    let bonding_curve_balance = client.get_balance(&Pubkey::from_str(bonding_curve).map_err(|_| actix_web::error::ErrorBadRequest("Invalid bonding curve pubkey format"))?).await.map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to get bonding curve balance: {}", e)))?;
                                                                    if bonding_curve_balance > 0 {
                                                                        println!("Premarket finished!Bonding Curve Account: {}, Balance: {} lamports", bonding_curve, bonding_curve_balance);
                                                                    } else {
                                                                        println!("Error! Bonding Curve Account {} has zero balance!", bonding_curve);
                                                                    }
                                                                }
                                                                if partially_decoded.data.starts_with("JcGQTTxFPsm") {
                                                                    //distribute
                                                                }
                                                                if partially_decoded.data.starts_with("2kHkz6JwH11") {
                                                                    //kill
                                                                }
                                                            }
                                                        },
                                                    }
                                                },
                                                solana_transaction_status::UiInstruction::Compiled(_) => {
                                                },
                                            };
                                        }
                                    }
                                }
                            }
                            solana_transaction_status::EncodedTransaction::LegacyBinary(_) => {
                            },
                            solana_transaction_status::EncodedTransaction::Binary(_, _) => {
                            },
                            solana_transaction_status::EncodedTransaction::Accounts(_) => {
                            }
                        } 
                    }
                    Ok(CheckTxResponse{
                        user_pubkey: None,
                        name: None,
                        symbol: None,
                        uri: None,
                        deadline: None,
                        goal_sol_lamp: None,
                        max_sol_lamp: None,
                        creator_allocate_lamp: None,
                        premarket: None,
                        lamports_in: None,
                    })
                },
                Err(e) => {
                    println!("Error fetching transaction: {}", e);
                    Ok(CheckTxResponse{
                        user_pubkey: None,
                        name: None,
                        symbol: None,
                        uri: None,
                        deadline: None,
                        goal_sol_lamp: None,
                        max_sol_lamp: None,
                        creator_allocate_lamp: None,
                        premarket: None,
                        lamports_in: None,
                    })
                }
            }
        },
        Some(_) => {
            println!("Transaction failed");
            Ok(CheckTxResponse{
                user_pubkey: None,
                name: None,
                symbol: None,
                uri: None,
                deadline: None,
                goal_sol_lamp: None,
                max_sol_lamp: None,
                creator_allocate_lamp: None,
                premarket: None,
                lamports_in: None,
            })
        },
        None => {
            println!("Transaction status is pending or not found");
            Ok(CheckTxResponse{
                user_pubkey: None,
                name: None,
                symbol: None,
                uri: None,
                deadline: None,
                goal_sol_lamp: None,
                max_sol_lamp: None,
                creator_allocate_lamp: None,
                premarket: None,
                lamports_in: None,
            })
        },
    }
}
