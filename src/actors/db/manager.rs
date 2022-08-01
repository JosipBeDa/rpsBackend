use super::messages::*;
use crate::{
    models::{
        hall_of_fame::NewHoFEntry, message::NewMessage, room::NewRoom,
        room_connection::NewRoomConnection,
    },
    state::db_pool,
};
use actix::prelude::*;
use colored::Colorize;
use tracing::info;

pub struct DBManager {
    db_pool: db_pool::PgPool,
}

impl DBManager {
    pub fn new(db_pool: db_pool::PgPool) -> Self {
        Self { db_pool }
    }
}

/// Make actor from `DBManager`
impl Actor for DBManager {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Context<Self>) {
        info!("{}", "Started DB Manager".green());
    }
}

impl Handler<StoreChatMessage> for DBManager {
    type Result = ();
    fn handle(&mut self, msg: StoreChatMessage, _: &mut Self::Context) -> Self::Result {
        let db_connection = self.db_pool.get().unwrap();
        NewMessage::store(&db_connection, &msg.message, msg.is_room)
            .expect("Couldn't store message");
    }
}

impl Handler<StoreRoom> for DBManager {
    type Result = ();
    fn handle(&mut self, msg: StoreRoom, _: &mut Self::Context) -> Self::Result {
        let db_connection = self.db_pool.get().unwrap();
        NewRoom::store(&db_connection, &msg.room, &msg.admin_id).expect("Couldn't store room");
    }
}

impl Handler<StoreRoomConnection> for DBManager {
    type Result = ();
    fn handle(&mut self, msg: StoreRoomConnection, _: &mut Self::Context) -> Self::Result {
        let db_connection = self.db_pool.get().unwrap();
        NewRoomConnection::set_connection(&db_connection, &msg.room_id, &msg.user_id)
            .expect("Couldn't store connection");
    }
}

impl Handler<StoreHoFEntry> for DBManager {
    type Result = ();
    fn handle(&mut self, msg: StoreHoFEntry, _: &mut Self::Context) -> Self::Result {
        let db_connection = self.db_pool.get().unwrap();
        NewHoFEntry::upsert(&db_connection, &msg.user_id).expect("Couldn't store HoF entry");
    }
}
