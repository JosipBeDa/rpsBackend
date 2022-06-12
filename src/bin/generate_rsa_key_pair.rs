#[allow(dead_code)]
use lib::crypto::gen_key_pair::generate_rsa_key_pair;

fn main() {
    generate_rsa_key_pair().expect("Couldn't generate keypair");
}