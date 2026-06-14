use crate::config::{get_pyth_mainnet_url, PYTH_MAINNET_URL_ENV};
use crate::models::premarket::PythResponse;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

pub const SOL_PRICE_CACHE_TTL: Duration = Duration::from_secs(60 * 10); // 10 minutes

#[derive(Debug, Clone)]
struct CacheEntry {
    value: f64,
    fetched_at: Instant,
}

// global cache
static SOL_PRICE_CACHE: RwLock<Option<CacheEntry>> = RwLock::const_new(None);

async fn fetch_sol_price_uncached(url: &str) -> f64 {
    match reqwest::get(url).await {
        Ok(resp) => match resp.json::<PythResponse>().await {
            Ok(pyth_response) => {
                if let Some(parsed) = pyth_response.parsed.first() {
                    let price_value: f64 = parsed.price.price.parse().unwrap_or(0.0);
                    price_value * 10_f64.powi(parsed.price.expo)
                } else {
                    0.0
                }
            }
            Err(_) => 0.0,
        },
        Err(_) => 0.0,
    }
}

// get with cache
pub async fn get_sol_price() -> f64 {
    let url = get_pyth_mainnet_url();
    if url == "" {
        println!("{} environment variable not set", PYTH_MAINNET_URL_ENV);
        return 0.0;
    };

    // read cache
    {
        let c = SOL_PRICE_CACHE.read().await;
        if let Some(e) = c.as_ref() {
            if e.fetched_at.elapsed() < SOL_PRICE_CACHE_TTL {
                return e.value;
            }
        }
    }

    // refresh (double-check)
    let mut c = SOL_PRICE_CACHE.write().await;
    if let Some(e) = c.as_ref() {
        if e.fetched_at.elapsed() < SOL_PRICE_CACHE_TTL {
            return e.value;
        }
    }

    let price = fetch_sol_price_uncached(&url).await;
    if price > 0.0 {
        *c = Some(CacheEntry {
            value: price,
            fetched_at: Instant::now(),
        });
    }
    price
}
