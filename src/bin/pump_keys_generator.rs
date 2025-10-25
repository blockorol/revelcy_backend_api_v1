use anyhow::{anyhow, Context, Result};
use solana_sdk::{pubkey::Pubkey, signature::{Keypair, SeedDerivable}};
use solana_sdk::signer::Signer;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{env, time::Duration};
use tokio::time::sleep;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use dotenv::dotenv;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use std::str::FromStr;

use revelcy_backend_api::storage::signing_keys::insert_mint_signing_key;



// MOVE CONST TO CONSTANTS FILE 
pub const MIN_KEYS: i64 = 100;              // maintain at least this many token keypairs 
pub const GRIND_THREADS: usize = 0;         // 0 = let solana-keygen auto-pick; else set e.g. 8

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

    let db_url = env::var("DATABASE_URL").context("DATABASE_URL not set")?;
    let target_suffix = "pu";

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .context("failed to connect to Postgres")?;

    let count = current_count(&pool).await?;
    info!("Current stored pump-keypairs: {}", count);

    if count < MIN_KEYS {
        let needed = (MIN_KEYS - count) as usize;
        info!("Need {} more with {}; starting grind loop…", needed, target_suffix);
        for i in 0..needed {
            info!("step {} of {};", i, needed);
            match grind_store_one(&pool, &target_suffix).await {
                Ok(pk) => info!("Stored key {} in step {}", pk, i),
                Err(e) => error!("Grind/store in step {i:#} failed: {e:#}"),
            }
        }
        info!("fihish generation for {needed:#} new keys");
    } else {
        info!("Enough keys in DB, nothing to do");
    }
    
    Ok(())
}

async fn current_count(pool: &PgPool) -> Result<i64> {
    let rec: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM signing_keys WHERE premarket_pubkey = 'does_not_exist'")
        .fetch_one(pool)
        .await?;
    Ok(rec.0)
}

async fn grind_store_one(pool: &PgPool, target_suffix: &str) -> Result<String> {
    // 1) генерируем подходящий ключ без внешнего процесса (и собираем метрики)
    let num_threads = if GRIND_THREADS > 0 { Some(GRIND_THREADS) } else { None };
    info!("Starting grind for suffix '{}' (threads: {})", target_suffix, num_threads.unwrap_or(0));
    let (kp, tries, elapsed) = grind_one_suffix(target_suffix, num_threads);

    let pubkey_b58 = kp.pubkey().to_string();
    if !pubkey_b58.ends_with(target_suffix) {
        return Err(anyhow!(
            "generated key does not end with '{}': {}",
            target_suffix,
            pubkey_b58
        ));
    }

    // Примерная скорость (keys/sec), защита от деления на 0
    let secs = elapsed.as_secs_f64().max(1e-6);
    let rate = (tries as f64) / secs;

    info!(
        "Vanity match found: pubkey={} (suffix='{}'), tries≈{}, time={:?}, speed≈{:.1} keys/sec",
        pubkey_b58, target_suffix, tries, elapsed, rate
    );

    // 2) 64-байтный секрет как строка для БД
    let sk64 = keypair_secret_64(&kp);
    let priv_key_str = sk64.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(",");

    // 3) Используем метку, а НЕ реальный premarket_account
    let premarket_account_str = "does_not_exist";

    // Лог: перед записью
    info!(
        "Storing key into DB: premarket_account='{}', pubkey={}",
        premarket_account_str, pubkey_b58
    );

    insert_mint_signing_key(
        pool,
        premarket_account_str,
        &pubkey_b58,
        &priv_key_str,
    ).await?;

    // Лог: успешная запись
    info!("DB insert OK for pubkey={}", pubkey_b58);

    Ok(pubkey_b58)
}

fn grind_one_suffix(target_suffix: &str, num_threads: Option<usize>) -> (Keypair, u64, Duration) {
    if let Some(n) = num_threads {
        let _ = rayon::ThreadPoolBuilder::new().num_threads(n.max(1)).build_global();
    }

    let found = Arc::new(AtomicBool::new(false));
    let attempts = Arc::new(AtomicU64::new(0));
    let started = Instant::now();

    let (kp, tries) = (0u64..u64::MAX)
        .into_par_iter()
        .find_map_any(|_| {
            if found.load(Ordering::Relaxed) {
                return None;
            }

            attempts.fetch_add(1, Ordering::Relaxed);

            let mut rng = ChaCha20Rng::from_entropy();
            let mut seed = [0u8; 32];
            rng.fill_bytes(&mut seed);

            // from_seed требует импорт SeedDerivable
            let kp = Keypair::from_seed(&seed).expect("seed -> keypair");
            let b58 = kp.pubkey().to_string();

            if b58.ends_with(target_suffix) {
                found.store(true, Ordering::Relaxed);
                let tries = attempts.load(Ordering::Relaxed);
                Some((kp, tries))
            } else {
                None
            }
        })
        .expect("should find a key");

    let elapsed = started.elapsed();
    (kp, tries, elapsed)
}

// Вспомогательная: собрать 64 байта секрета как в solana keypair json
fn keypair_secret_64(kp: &Keypair) -> [u8; 64] {
    // solana-sdk хранит ed25519: 32 байта секрета + 32 байта паблика
    // Keypair::to_bytes() уже даёт нужные 64 байта
    kp.to_bytes()
}

