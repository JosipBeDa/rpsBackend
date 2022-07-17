use crate::models::error::{AuthenticationError, GlobalError};
use crate::models::user::ChatUser;
use crate::TOKEN_DURATION;
use colored::Colorize;
use jsonwebtoken::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::info;

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
pub fn generate_jwt(serialized_user: String) -> Result<String, GlobalError> {
    let priv_key = fs::read(Path::new("./key_pair/priv_key.pem"))?;
    let encoding_key = EncodingKey::from_rsa_pem(&priv_key)?;
    let now = jsonwebtoken::get_current_timestamp();
    let exp_timestamp = now + TOKEN_DURATION.whole_seconds() as u64;
    info!(
        "{}{}{}{}",
        "Creating JWT -- Now: ".cyan(),
        now,
        " Exp: ".cyan(),
        exp_timestamp
    );
    let claims = Claims::new(serialized_user, now, exp_timestamp);
    let token = encode(&Header::new(Algorithm::RS256), &claims, &encoding_key)?;
    Ok(token)
}

/// Verifies the token issued by the generate_jwt token.
pub fn verify(token: &str) -> Result<ChatUser, GlobalError> {
    info!("{}", "Verifying JWT".cyan());
    // Fetch public key
    let pub_key = fs::read(Path::new("./key_pair/pub_key.pem"))?;
    let decoding_key = DecodingKey::from_rsa_pem(&pub_key)?;
    let token_data =
        jsonwebtoken::decode::<Claims>(&token, &decoding_key, &Validation::new(Algorithm::RS256))?;
    // Check if the token expired
    if token_data.claims.exp < jsonwebtoken::get_current_timestamp() {
        return Err(AuthenticationError::InvalidToken.into());
    }
    let user: ChatUser = serde_json::from_str(&token_data.claims.sub)?;
    Ok(user)
}
