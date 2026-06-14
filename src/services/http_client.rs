use reqwest::Client;
use std::time::Duration;

pub const DEFAULT_HTTP_TIMEOUT: Duration = Duration::from_secs(30);
pub const LONG_HTTP_TIMEOUT: Duration = Duration::from_secs(90);

pub fn default_client() -> Result<Client, reqwest::Error> {
    client_with_timeout(DEFAULT_HTTP_TIMEOUT)
}

pub fn long_timeout_client() -> Result<Client, reqwest::Error> {
    client_with_timeout(LONG_HTTP_TIMEOUT)
}

fn client_with_timeout(timeout: Duration) -> Result<Client, reqwest::Error> {
    Client::builder().timeout(timeout).build()
}
