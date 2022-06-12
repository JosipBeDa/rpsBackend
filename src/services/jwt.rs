use crate::models::authentication::Claims;
use crate::models::custom_error::CustomError;
use jsonwebtoken::*;


pub fn generate_jwt<'a>(user_id: &'a str, priv_key: &Vec<u8>) -> Result<(String, u64), CustomError> {
    let encoding_key =
        EncodingKey::from_rsa_pem(priv_key)?;
    let now = jsonwebtoken::get_current_timestamp();
    let expires = now + 60 * 5;
    let claims = Claims::new(user_id.to_string(), now, expires);
    let token = encode(&Header::new(Algorithm::RS256), &claims, &encoding_key)?;
    Ok((token, expires))
}
