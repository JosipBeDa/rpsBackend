use rand;
use rsa::{
    pkcs1::EncodeRsaPublicKey, pkcs8::EncodePrivateKey, pkcs8, pkcs1, RsaPrivateKey,
    RsaPublicKey,
};
use std::path;

enum WriteError {
    Pkcs8(rsa::pkcs8::Error),
    Pkcs1(rsa::pkcs1::Error),
}

fn gen_key_pair() -> Result<(), WriteError> {
    let mut rng = rand::thread_rng();
    let bits = 2048;
    let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("Failed to generate private key");
    let pub_key = RsaPublicKey::from(&priv_key);
    if let Err(e) = priv_key.write_pkcs8_pem_file(path::Path::new("./priv_key.pem"), pkcs8::LineEnding::LF)
    {
        return Err(WriteError::Pkcs8(e));
    }
    if let Err(e) = pub_key.write_pkcs1_pem_file(path::Path::new("./pub_key.pem"), pkcs1::LineEnding::LF) {
        return Err(WriteError::Pkcs1(e));
    }
    Ok(())
}
