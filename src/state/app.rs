use crate::chat::rps::RPSManager;
use crate::chat::server::ChatServer;
use crate::state::client;
use crate::state::db_pool;
use actix::{Actor, Addr};

#[derive(Clone)]
pub struct AppState {
    pub client: reqwest::Client,
    pub db_pool: db_pool::PgPool,
    pub chat_server: Addr<ChatServer>,
    pub rps_manager: Addr<RPSManager>,
}

impl AppState {
    pub fn initialize() -> Self {
        let db_pool = db_pool::establish_pool_connection();
        let client = client::initialize();
        let chat_server = ChatServer::new().start();
        let rps_manager = RPSManager::new().start();
        AppState {
            client,
            db_pool,
            chat_server,
            rps_manager
        }
    }
}
