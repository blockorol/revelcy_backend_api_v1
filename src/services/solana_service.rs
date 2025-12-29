use anyhow::{Context, anyhow, Result};
use bincode;
use bs58;
use sha2::Sha256;
use solana_client::nonblocking::rpc_client::RpcClient as AsyncRpcClient;
use solana_sdk::system_program::ID as SYSTEM_PROGRAM_ID;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    hash::Hash,
    compute_budget::sign_tx_with_revelcy,
    instruction::{AccountMeta, Instruction},
    message::Message, pubkey::Pubkey,
    signature::{read_keypair_file, Keypair},
    transaction::Transaction
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use std::{time::Duration, path::Path, str::FromStr};
use solana_transaction_status::UiTransactionEncoding;

use crate::models::premarket::{
    BuildKillTxParams, 
    CheckTxParams,
    GetPremarketDataParams, 
    DistributeTokensParams, 
    PremarketOnchainUser,
    PremarketOnchainData,
    SolanaNetwork, 
    DeployTxParams,
};

use crate::api::premarket::CheckTxResponse;

use serde_json;
use sqlx::PgPool;

use spl_associated_token_account::ID as associated_token_program_id;
use spl_associated_token_account::get_associated_token_address;
use spl_token::ID as token_program_id;

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

const KILL_METHOD_NAME: &str = "kill_premarket";
const DISTRIBUTE_METHOD_NAME: &str = "distribute_tokens";

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
        SolanaNetwork::Devnet      => ("PURPLE_PROGRAM_ID_DEV",  "AUf85EmXsTYGGgQnYJR2heKCxkTf5WtnKFvpkLtQ58sG"),
        SolanaNetwork::MainnetBeta => ("PURPLE_PROGRAM_ID_MAIN", "AtuMMXXjyAW3fSJrnWYon1ynxUA7CyQ3Qz2E6JqiTmhu"),
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

// todo: move to service v2
pub fn sign_tx_with_revelcy(
    tx_base64: &str,
    network: SolanaNetwork,
    extra_signers: Option<&[Keypair]>,
) -> Result<String> {
    let revelcy = read_revelcy_auth(network);

    // 1) Декодим вход (уже частично/полностью подписанную пользователем транзу)
    let raw = BASE64
        .decode(tx_base64)
        .context("BASE64 decode(tx_base64) failed")?;
    let mut tx: Transaction =
        bincode::deserialize(&raw).context("bincode deserialize(Transaction) failed")?;

    let blockhash = tx.message.recent_blockhash;

    // 2) Собираем список всех подписантов: revelcy + (опционально) доп. ключи
    let mut signers: Vec<&Keypair> = Vec::new();
    signers.push(&revelcy);

    if let Some(extra) = extra_signers {
        for kp in extra {
            signers.push(kp);
        }
    }

    // 3) Частичная подпись всеми ключами
    tx.try_partial_sign(&signers, blockhash)
        .context("failed to partially sign transaction with revelcyAuth and extra_signers")?;

    // 4) Сериализация обратно в base64
    let signed_raw =
        bincode::serialize(&tx).context("bincode serialize(signed Transaction) failed")?;
    let signed_b64 = BASE64.encode(signed_raw);

    Ok(signed_b64)
}


pub async fn distribute_tk(
    _pool: &PgPool,
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
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
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

    let _discriminator: [u8; 8] = [
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

pub async fn test_build_kill_premarket_tx(
    _pool: &PgPool,
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
    let all_entered_users = params.users;

    let mut accounts = vec![
        AccountMeta::new(revelcy_pub, true),
        AccountMeta::new(premarket_account, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
    ];

    for user in all_entered_users {
        accounts.push(AccountMeta::new(Pubkey::from_str(&user).unwrap(), false));
    }

    let _discriminator: [u8; 8] = [
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
    // Format: 4 bytes for length + (users_len * (32 bytes for pubkey + 8 bytes for lamports + 1 byte for claimed))
    let offset_after_users = 4 + (users_len as usize * 41);
    let mut all_entered_users = Vec::<PremarketOnchainUser>::new();

    // Extract user data if available
    if users_len > 0 {
        for i in 0..users_len as usize {
            let user_offset = 4 + (i * 41); // 4 bytes for vector length + i * entry size
            let pubkey_bytes = &data[user_offset..user_offset+32];
            let pubkey = Pubkey::new_from_array(pubkey_bytes.try_into()?);

            let lamports_bytes = &data[user_offset+32..user_offset+40];
            let lamports = u64::from_le_bytes(lamports_bytes.try_into()?);

            let claimed = data[user_offset+40] != 0;

            let user = PremarketOnchainUser {
                wallet: pubkey,
                contributed_lamports: lamports,
                claimed,
            };
            all_entered_users.push(user);

            println!("User {}: {} contributed {} lamports, claimed: {}", i, pubkey, lamports, claimed);
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
        extended_premarket: end_timestamp_updated,
        goal_lamports: goal_sol,
        max_lamports: max_sol,
        mint,
    })
}

// !!!now user is CONST i need to change it later!!!
pub async fn deploy_tx_service(
    _pool: &PgPool,
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

    println!("recent_blockhash: {:?}", recent_blockhash);

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

