use serde::Deserialize;
use std::error::Error;
use std::fmt;

use crate::models::premarket::{TokenInfo, TokenLinks};

const REVELCY_SUFFIX: &str =
    "Premarket done with Revelcy; initial buy distributed to the community. More: beta.revelcy.com";

#[derive(Debug)]
pub enum IpfsServiceError {
    Fetch(reqwest::Error),
    HttpStatus(reqwest::StatusCode),
    InvalidJson(reqwest::Error),
}

impl fmt::Display for IpfsServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fetch(err) => write!(f, "failed to fetch metadata: {err}"),
            Self::HttpStatus(status) => write!(f, "metadata fetch failed: http {status}"),
            Self::InvalidJson(err) => write!(f, "invalid metadata json: {err}"),
        }
    }
}

impl Error for IpfsServiceError {}

#[derive(Debug, Deserialize)]
struct IpfsMetadata {
    name: Option<String>,
    symbol: Option<String>,
    description: Option<String>,
    image: Option<String>,
    // rare case with collection
    external_url: Option<String>,

    // "properties": { "links": { "twitter": "...", ... } }
    properties: Option<IpfsProperties>,
}

#[derive(Debug, Deserialize)]
struct IpfsProperties {
    links: Option<IpfsLinks>,
}

#[derive(Debug, Deserialize)]
struct IpfsLinks {
    telegram: Option<String>,
    twitter: Option<String>,
    web_site: Option<String>,
    website: Option<String>,
}

fn ipfs_to_gateway_url(s: &str) -> String {
    // can be chaged to other gateway if needed
    const GW: &str = "https://ipfs.io/ipfs/";

    if let Some(rest) = s.strip_prefix("ipfs://") {
        // ipfs://CID/path -> https://ipfs.io/ipfs/CID/path
        format!("{GW}{rest}")
    } else if let Some(rest) = s.strip_prefix("ipfs:/") {
        // for case ipfs:/CID
        format!("{GW}{}", rest.trim_start_matches('/'))
    } else {
        s.to_string()
    }
}

pub async fn get_ipfs_token_info(uri: &String) -> Result<TokenInfo, IpfsServiceError> {
    // 1) convert uri to normal (for ipfs://)
    let url = ipfs_to_gateway_url(uri.as_str());

    // 2) get metadata
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(IpfsServiceError::Fetch)?;

    if !resp.status().is_success() {
        return Err(IpfsServiceError::HttpStatus(resp.status()));
    }

    // 3) parce JSON
    let meta: IpfsMetadata = resp.json().await.map_err(IpfsServiceError::InvalidJson)?;

    let external_url = meta.external_url.clone();
    // 4) links
    let links = meta
        .properties
        .and_then(|p| p.links)
        .map(|l| TokenLinks {
            telegram: l.telegram,
            twitter: l.twitter,
            web_site: l.web_site.or(l.website).or(external_url.clone()),
        })
        .unwrap_or(TokenLinks {
            telegram: None,
            twitter: None,
            web_site: external_url,
        });

    // 5) image_url (also can be ipfs://)
    let image_url = meta.image.map(|img| ipfs_to_gateway_url(img.as_str()));
    let raw_description = meta.description.unwrap_or_default();

    let description = raw_description
        .trim_end()
        .strip_suffix(REVELCY_SUFFIX)
        .map(|s| s.trim_end().to_string())
        .unwrap_or(raw_description);

    Ok(TokenInfo {
        address: "".to_string(), // empty, because it's not from IPFS
        name: meta.name.unwrap_or_default(),
        description: description,
        symbol: meta.symbol.unwrap_or_default(),
        image_url: image_url,
        data_uri: uri.clone(),
        links,
    })
}
