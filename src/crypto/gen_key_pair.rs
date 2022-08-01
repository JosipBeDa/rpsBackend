use rand;
use rsa::{
    pkcs1, pkcs1::EncodeRsaPublicKey, pkcs8, pkcs8::EncodePrivateKey, RsaPrivateKey, RsaPublicKey,
};
use tracing::info;
use tracing::log::warn;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub enum WriteError {
    FileSystemError(std::io::Error),
    Pkcs8(rsa::pkcs8::Error),
    Pkcs1(rsa::pkcs1::Error),
}

/// Generates an 2048 bit RSA key pair and stores in the root directory in a directory named key_pair.
pub fn generate_rsa_key_pair() -> Result<(), WriteError> {
    let mut rng = rand::thread_rng();
    let bits = 2048;
    let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("Failed to generate private key");
    let pub_key = RsaPublicKey::from(&priv_key);
    info!("Attempting to remove 'key_pair' directory");
    match fs::remove_dir_all(Path::new("./key_pair")) {
        Ok(()) => info!("Deleting old key_pair directory"),
        Err(e) => warn!("{}.", e),
    }
    info!("Creating new directory 'key_pair'");
    if let Err(e) = fs::create_dir(Path::new("./key_pair")) {
        return Err(WriteError::FileSystemError(e));
    }
    if let Err(e) =
        priv_key.write_pkcs8_pem_file(Path::new("./key_pair/priv_key.pem"), pkcs8::LineEnding::LF)
    {
        return Err(WriteError::Pkcs8(e));
    }
    if let Err(e) =
        pub_key.write_pkcs1_pem_file(Path::new("./key_pair/pub_key.pem"), pkcs1::LineEnding::LF)
    {
        return Err(WriteError::Pkcs1(e));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{services::jwt::Claims, actors::chat::models::chat_user::ChatUser};
    use jsonwebtoken::*;

    #[test]
    fn encode_decode_jwt() {
        //Fetch the private key
        let priv_key =
            fs::read(Path::new("./key_pair/priv_key.pem")).expect("Couldn't open private key");
        //Fetch the public key
        let pub_key =
            fs::read(Path::new("./key_pair/pub_key.pem")).expect("Couldn't open private key");

        //Transmogrify the key key par to the encoding and decoding keys as arrays of u8
        let encoding_key = jsonwebtoken::EncodingKey::from_rsa_pem(&priv_key)
            .expect("Couldn't parse encoding key");
        let decoding_key =
            jsonwebtoken::DecodingKey::from_rsa_pem(&pub_key).expect("Couldn't parse decoding key");

        //Issued at
        let now = jsonwebtoken::get_current_timestamp();
        //Expires in 5 minutes
        let expires = now + 60 * 5;
        //Generate the claims
        let user = ChatUser {id: String::from("lol"), username: String::from("lawl"), connected: false};
        let claims = Claims::new(user.to_string(), now, expires);
        eprintln!("{claims:?}");
        //Encode jwt
        let token = encode(&jsonwebtoken::Header::new(Algorithm::RS256), &claims, &encoding_key)
            .expect("Couldn't encode token");
        eprintln!("token: {}", token);
        //Set headers for decoding
        let validation = Validation::new(Algorithm::RS256);

        //Decode the token
        let decoded = decode::<Claims>(&token, &decoding_key, &validation).expect("Couldn't decode token");

        assert_eq!(claims, decoded.claims);
        assert_eq!(expires, decoded.claims.exp);
        assert_eq!(now, decoded.claims.iat);
        assert_eq!(Algorithm::RS256, decoded.header.alg);
        //assert_eq!(claims.sub, decoded.claims.sub)
    }
}
