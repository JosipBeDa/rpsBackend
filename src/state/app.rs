use crate::state::client;
use crate::state::db_pool;
use std::fs;
use std::path::Path;

#[derive(Clone)]
pub struct AppState {
    pub client: reqwest::Client,
    pub db_pool: db_pool::PgPool,
    pub priv_key: Vec<u8>,
}

impl AppState {
    pub fn initialize() -> Self {
        let db_pool = db_pool::establish_pool_connection();
        let client = client::initialize();
        let priv_key = fs::read(Path::new("./key_pair/priv_key.pem")).expect("Couldn't read private key");
        AppState {
            client,
            db_pool,
            priv_key,
        }
    }
}
