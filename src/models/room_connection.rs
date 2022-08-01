use crate::schema::room_connections;
use diesel::{PgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};

use super::error::GlobalError;
#[derive(Queryable, Debug, Clone, Serialize, Deserialize)]
pub struct RoomConnection {
    room_id: String,
    user_id: String,
}
#[derive(Insertable, Debug, AsChangeset)]
#[table_name = "room_connections"]
pub struct NewRoomConnection<'a> {
    room_id: &'a str,
    user_id: &'a str,
}

impl<'a> NewRoomConnection<'a> {
    pub fn set_connection(
        conn: &PgConnection,
        room_id: &'a str,
        user_id: &'a str,
    ) -> Result<usize, GlobalError> {
        let connection = Self { room_id, user_id };
        diesel::insert_into(room_connections::table)
            .values(connection)
            .on_conflict_do_nothing()
            .execute(conn)
            .map_err(|e| GlobalError::DieselError(e))
    }
}
