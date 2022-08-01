use crate::actors::chat::models::messages::ChatMessage;
use crate::schema::messages;
use chrono::{DateTime, Local};
use diesel::{prelude::*, Insertable, Queryable, RunQueryDsl};
use serde::{Deserialize, Serialize};

#[derive(Queryable, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    id: String,
    sender_id: String,
    receiver_user: Option<String>,
    receiver_room: Option<String>,
    content: String,
    timestamp: DateTime<Local>,
}
#[derive(Insertable, Debug)]
#[table_name = "messages"]
pub struct NewMessage<'a> {
    id: &'a str,
    sender_id: &'a str,
    receiver_user: Option<&'a str>,
    receiver_room: Option<&'a str>,
    content: &'a str,
    timestamp: DateTime<Local>,
}

impl<'a> NewMessage<'a> {
    pub fn store(
        conn: &PgConnection,
        message: &'a ChatMessage,
        room: bool,
    ) -> Result<usize, diesel::result::Error> {
        let new_message = Self {
            id: &message.id,
            sender_id: &message.sender_id,
            receiver_user: if !room {
                Some(&message.receiver_id)
            } else {
                None
            },
            receiver_room: if room {
                Some(&message.receiver_id)
            } else {
                None
            },
            content: &message.content,
            timestamp: Local::now(),
        };
        diesel::insert_into(messages::table)
            .values(new_message)
            .execute(conn)
    }
}
