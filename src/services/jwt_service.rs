use crate::config;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub nonce: String,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenWithUserInfo {
    pub sub: String, // same to user_id
    pub user_id: Uuid,
    pub username: String,
    pub avatar_url: Option<String>,
    pub current_wallet: Option<String>,
    pub nonce: String,
    pub exp: usize,
}

pub fn create_jwt_handle(nonce: &str) -> String {
    let secret = config::get_jwt_secret();
    let claims = Claims {
        nonce: nonce.to_owned(),
        exp: 2000000000, // заглушка
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .unwrap()
}

pub fn create_jwt_with_user(
    user_id: Uuid,
    user_wallet: &str,
    username: Option<String>,
    avatar_url: Option<String>,
    nonce: &str,
) -> String {
    let secret = config::get_jwt_secret();

    let token_info: TokenWithUserInfo = TokenWithUserInfo {
        user_id: user_id,
        sub: user_id.to_string(),
        current_wallet: Some(user_wallet.to_string()),
        username: username.unwrap_or_default(),
        avatar_url,
        nonce: nonce.to_owned(),
        exp: 2000000000,
    };
    encode(
        &Header::default(),
        &token_info,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .unwrap()
}

pub fn decode_jwt_handle(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = config::get_jwt_secret();

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    )?;
    Ok(token_data.claims)
}

pub fn decode_jwt_with_user_info(
    token: &str,
) -> Result<TokenWithUserInfo, jsonwebtoken::errors::Error> {
    let secret = config::get_jwt_secret();
    let token_data = decode::<TokenWithUserInfo>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    )?;
    Ok(token_data.claims)
}
