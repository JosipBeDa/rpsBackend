use crate::models::authentication::AuthenticationError;
use crate::models::custom_error::CustomError;
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
        let iat = iat;
        let exp = exp;
        Self { sub, iat, exp }
    }
}

pub fn generate_jwt<'a>(user_id: &'a str) -> Result<(String, u64), CustomError> {
    let priv_key = fs::read(Path::new("./key_pair/priv_key.pem"))?;
    let encoding_key = EncodingKey::from_rsa_pem(&priv_key)?;
    let now = jsonwebtoken::get_current_timestamp();
    let expires = now + 60 * 60 * 24;
    let claims = Claims::new(user_id.to_string(), now, expires);
    let token = encode(&Header::new(Algorithm::RS256), &claims, &encoding_key)?;
    Ok((token, expires))
}

pub fn verify<'a>(token: &'a str) -> Result<String, CustomError> {
    // Fetch public key
    let pub_key = fs::read(Path::new("./key_pair/pub_key.pem"))?;
    let decoding_key = DecodingKey::from_rsa_pem(&pub_key)?;
    
    let token_data =
        jsonwebtoken::decode::<Claims>(token, &decoding_key, &Validation::new(Algorithm::RS256))?;
    // Check if the token expired
    if token_data.claims.exp < jsonwebtoken::get_current_timestamp() {
        return Err(AuthenticationError::TokenExpired.into());
    }
    Ok(token_data.claims.sub)
}
