use crate::models::authentication::AuthenticationError;
use crate::models::custom_error::CustomError;
use crate::models::user::ChatUser;
use cookie::time::Duration;
use jsonwebtoken::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Used for jwts. sub is the actual payload, iat and exp are unix timestamps representing the issued at and expiration times respectively.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iat: u64,
    pub exp: u64,
}

impl Claims {
    pub fn new(sub: String, iat: u64, exp: u64) -> Self {
        Self { sub, iat, exp }
    }
}

/// Generates a JWT using the RS256 algorithm
pub fn generate_jwt(serialized_user: String) -> Result<(String, Duration), CustomError> {
    let priv_key = fs::read(Path::new("./key_pair/priv_key.pem"))?;
    let encoding_key = EncodingKey::from_rsa_pem(&priv_key)?;
    let now = jsonwebtoken::get_current_timestamp();
    let expiration_timestamp = now + 60 * 60 * 2;
    let expires = Duration::hours(2);
    let claims = Claims::new(serialized_user, now, expiration_timestamp);
    let token = encode(&Header::new(Algorithm::RS256), &claims, &encoding_key)?;
    Ok((token, expires))
}

/// Verifies the token issued by the generate_jwt token.
pub fn verify(token: &str) -> Result<ChatUser, CustomError> {
    // Fetch public key
    let pub_key = fs::read(Path::new("./key_pair/pub_key.pem"))?;
    let decoding_key = DecodingKey::from_rsa_pem(&pub_key)?;

    let token_data =
        jsonwebtoken::decode::<Claims>(&token, &decoding_key, &Validation::new(Algorithm::RS256))?;
    // Check if the token expired
    if token_data.claims.exp < jsonwebtoken::get_current_timestamp() {
        return Err(AuthenticationError::TokenExpired.into());
    }
    let user: ChatUser = serde_json::from_str(&token_data.claims.sub)?;
    Ok(user)
}
