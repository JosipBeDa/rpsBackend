use std::pin::Pin;

use super::{client, db_pool};
use crate::actors::chat::server::ChatServer;
use crate::actors::{db::manager::DBManager, rps::manager::RPSManager};
use actix::{Actor, Addr};

#[derive(Clone)]
pub struct AppState {
    pub client: reqwest::Client,
    pub db_pool: db_pool::PgPool,
    pub chat_server: Addr<ChatServer>,
    pub rps_manager: Addr<RPSManager>,
    pub db_manager: Addr<DBManager>,
}

impl AppState {
    pub fn initialize() -> Self {
        let db_pool = db_pool::establish_pool_connection();
        let client = client::initialize();
        let db_manager = DBManager::new(db_pool.clone()).start();
        let chat_server = ChatServer::new(Pin::new(&db_manager).get_ref().clone()).start();
        let rps_manager = RPSManager::new(Pin::new(&db_manager).get_ref().clone()).start();
        AppState {
            client,
            db_pool,
            chat_server,
            rps_manager,
            db_manager,
        }
    }
}
