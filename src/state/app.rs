use crate::state::client;
use crate::state::db_pool;

#[derive(Clone)]
pub struct AppState {
    pub client: reqwest::Client,
    pub db_pool: db_pool::PgPool,
}

impl AppState {
    pub fn initialize() -> Self {
        let db_pool = db_pool::establish_pool_connection();
        let client = client::initialize();
        AppState { client, db_pool }
    }
}
