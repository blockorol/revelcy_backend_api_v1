use actix_web::{web, HttpResponse, Scope};
use sqlx::PgPool;
use crate::api::dto::*;
use crate::models::user::WalletAddress;
use crate::services::auth_service;
use crate::services::jwt_service;
use crate::services::wallet_service::{get_signatures_for_wallet, get_creation_time};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

pub fn public_scope() -> Scope {
    web::scope("/auth")
        .route("/start_session", web::get().to(start_session))
        .route("/confirm_login", web::post().to(confirm_login))
        .route("/wallet_info/{pubkey}", web::get().to(wallet_info))
        .route("/premarket_info/{premarket_account}", web::get().to(premarket_info))
}

async fn start_session() -> HttpResponse {
    let session = auth_service::generate_session();
    let jwt = jwt_service::create_jwt_handle(&session.nonce);

    let response = StartSessionResponseDto {
        nonce: session.nonce,
        jwt,
    };
    HttpResponse::Ok().json(response)
}

pub async fn confirm_login(
    pool: web::Data<PgPool>,
    req: web::Json<ConfirmLoginRequestDto>,
) -> HttpResponse {
    let handle = match jwt_service::decode_jwt_handle(&req.jwt) {
        Ok(handle) => handle,
        Err(_) =>  {
            return HttpResponse::Unauthorized().body("no jwt");
        }
    };
    let nonce = handle.nonce;

    let wallet = WalletAddress {
        address: req.wallet_address.clone(),
    };

    if !auth_service::verify_signature(&wallet, &nonce, &req.signature) {
        return HttpResponse::Unauthorized().body("Invalid signature");
    }

    let (user, is_new_user) = match auth_service::get_or_create_user_info(pool, &wallet.address).await {
        Ok((user, is_new_user)) => (user, is_new_user),
        Err(err_msg) => return HttpResponse::InternalServerError().body(err_msg),
    };

    let token = jwt_service::create_jwt_with_user(
        user.id,
        &req.wallet_address,
        user.username.clone(),
        user.avatar_url.clone(),
        &nonce
    );

    let response = ConfirmLoginResponseDto {
        jwt: token,
        is_new_user: is_new_user
    };
    HttpResponse::Ok().json(response)
}

async fn wallet_info(
    pubkey: web::Path<String>,
) -> HttpResponse {
    let rpc_url = match std::env::var("SOLANA_DEVNET_RPC") {
        Ok(url) => url,
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "SOLANA_DEVNET_RPC environment variable not set"
        })),
    };
    
    let pubkey_str = pubkey.as_str().to_string();
    
    // Use web::block to run blocking RPC operations in a thread pool
    let result = web::block(move || {
        let pubkey_parsed = match Pubkey::from_str(&pubkey_str) {
            Ok(key) => key,
            Err(_) => return Err("Invalid wallet address format".to_string()),
        };
        
        let rpc_client = RpcClient::new(rpc_url);
        
        // Get balance
        let balance_lamports = match rpc_client.get_balance(&pubkey_parsed) {
            Ok(balance) => balance,
            Err(e) => return Err(format!("Failed to get balance: {}", e)),
        };
        
        // Get signatures
        //let all_signatures = get_signatures_for_wallet(&rpc_client, &pubkey_parsed);
        let all_signatures: Vec<String> = Vec::new(); // TODO: remove this
        // Get creation time with error handling
        //let creation_time = get_creation_time(&rpc_client, &all_signatures).unwrap().to_string();
        let creation_time = "2025-01-01".to_string(); // TODO: remove this

        let balance = format!("{:.2}", (balance_lamports as f64) / 1_000_000_000.0);
        println!("Wallet balance: {}", balance);
        let balance_parsed = balance.parse::<f64>().unwrap_or(0.0);
        
        Ok((creation_time, balance_parsed, all_signatures.len().to_string()))
    })
    .await;
    
    // Handle the nested Result correctly
    match result {
        Ok(inner_result) => match inner_result {
            Ok((creation_time, balance, tx_amount)) => {
                let response = WalletInfoResponseDto {
                    creation_time,
                    balance,
                    tx_amount,
                };
                HttpResponse::Ok().json(response)
            },
            Err(e) => {
                eprintln!("Error in wallet processing: {}", e);
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": e
                }))
            }
        },
        Err(e) => {
            eprintln!("Error executing blocking operation: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to process wallet information due to server error"
            }))
        }
    }
}

async fn premarket_info(
    premarket_account_str: web::Path<String>,
) -> HttpResponse {
    let rpc_url = match std::env::var("RPC_URL") {
        Ok(url) => url,
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "RPC_URL environment variable not set"
        })),
    };
    
    let account_str = premarket_account_str.as_str().to_string();
    
    let result = web::block(move || {
        let premarket_account = match Pubkey::from_str(&account_str) {
            Ok(key) => key,
            Err(_) => return Err("Invalid premarket account address format".to_string()),
        };

        let client = RpcClient::new(rpc_url);

        let account = match client.get_account(&premarket_account) {
            Ok(account) => account,
            Err(e) => return Err(format!("Failed to get account: {}", e)),
        };
        
        println!("Account owner: {}", account.owner);
        println!("Account data length: {} bytes", account.data.len());
        
        // Skip the 8-byte discriminator
        let data = &account.data[8..];

        // Initialize offset for tracking position in data
        let mut offset = 0;

        // Get the user vector length
        let users_len = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap());
        println!("\nUsers vector length: {}", users_len);
        offset += 4;
        
        let mut users = Vec::new();

        // Extract user data if available
        if users_len > 0 {
            for _ in 0..users_len as usize {
                // Extract pubkey (32 bytes)
                let pubkey_bytes = &data[offset..offset+32];
                let pubkey = Pubkey::new_from_array(pubkey_bytes.try_into().unwrap()).to_string();
                offset += 32;
                
                // Extract lamports (8 bytes)
                let lamports_bytes = &data[offset..offset+8];
                let lamports = u64::from_le_bytes(lamports_bytes.try_into().unwrap());
                offset += 8;

                users.push((pubkey, lamports));
            }
        }
        
        // Extract end_timestamp (8 bytes)
        let end_timestamp = i64::from_le_bytes(data[offset..offset+8].try_into().unwrap());
        offset += 8;
        
        // Extract goal_sol (8 bytes)
        let goal_sol = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap());
        offset += 8;
        
        // Extract max_sol (8 bytes)
        let max_sol = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap());
        offset += 8;
        
        // Extract mint (32 bytes Pubkey)
        let mint = Pubkey::new_from_array(data[offset..offset+32].try_into().unwrap()).to_string();
        offset += 32;
        
        // Extract name (4 bytes length + variable bytes)
        let name_len = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
        offset += 4;
        let name = std::str::from_utf8(&data[offset..offset+name_len])
            .map_err(|_| "Invalid UTF-8 in name".to_string())?
            .to_string();
        offset += name_len;
        
        // Extract symbol (4 bytes length + variable bytes)
        let symbol_len = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
        offset += 4;
        let symbol = std::str::from_utf8(&data[offset..offset+symbol_len])
            .map_err(|_| "Invalid UTF-8 in symbol".to_string())?
            .to_string();
        offset += symbol_len;
        
        // Extract uri (4 bytes length + variable bytes)
        let uri_len = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
        offset += 4;
        let uri = std::str::from_utf8(&data[offset..offset+uri_len])
            .map_err(|_| "Invalid UTF-8 in uri".to_string())?
            .to_string();
        offset += uri_len;
        
        // Extract creator (32 bytes Pubkey)
        let creator = Pubkey::new_from_array(data[offset..offset+32].try_into().unwrap()).to_string();
        
        let response = PremarketInfoResponseDto {
            users,
            end_timestamp,
            goal_sol,
            max_sol,
            mint,
            name,
            symbol,
            uri,
            creator,
        };
        Ok(response)
    })
    .await;
    
    match result {
        Ok(inner_result) => match inner_result {
            Ok(response) => HttpResponse::Ok().json(response),
            Err(e) => {
                eprintln!("Error in premarket processing: {}", e);
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": e
                }))
            }
        },
        Err(e) => {
            eprintln!("Error executing blocking operation: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to process premarket information due to server error"
            }))
        }
    }
}