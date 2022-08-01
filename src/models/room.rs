use crate::{actors::chat::models::room::PublicRoom, schema::rooms};
use diesel::{Insertable, PgConnection, Queryable, RunQueryDsl};
use serde::{Deserialize, Serialize};

use super::error::GlobalError;

#[derive(Queryable, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    id: String,
    name: String,
    password: Option<String>,
    admin: String,
}

#[derive(Insertable, Debug)]
#[table_name = "rooms"]
pub struct NewRoom<'a> {
    id: &'a str,
    name: &'a str,
    password: Option<&'a str>,
    admin: &'a str,
}

impl<'a> NewRoom<'a> {
    pub fn store(
        conn: &PgConnection,
        room: &'a PublicRoom,
        admin: &'a str,
    ) -> Result<usize, GlobalError> {
        let new_room = Self {
            id: &room.id,
            name: &room.name,
            password: Some("To do"),
            admin,
        };
        diesel::insert_into(rooms::table)
            .values(new_room)
            .execute(conn)
            .map_err(|e| GlobalError::DieselError(e))
    }
}
