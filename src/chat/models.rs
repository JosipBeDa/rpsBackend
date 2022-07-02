use actix::prelude::*;
use serde::{Deserialize, Serialize};

/// Chat server sends these messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

// Messages for chat server communications

/// Creates a new chat session. Always the first message sent from the client.
/// `addr` - the address of the recepient
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<Message>,
}

/// Session is disconnected
///
/// `id`: The ID of the session to disconnect
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub session_id: String,
}

/// Send message to specific room
///
/// `id`: the client session ID,
/// `msg`: the text content of the message,
/// `room`: the room name
#[derive(Message, Debug, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub session_id: String,
    pub header: String,
    pub body: Option<String>,
    pub data: Option<MessageData>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageData {
    String(String),
    List(Vec<String>)
}

impl ToString for ClientMessage {
    fn to_string(&self) -> String {
        serde_json::to_string(self).expect("Couldn't serialize struct")
    }
}

#[derive(Message, Debug, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct ChatMessage {
    pub id: String,
    pub sender_id: String,
    pub receiver_id: String,
    pub content: String,
    pub read: bool,
}

/// Lists available rooms
pub struct ListRooms;
impl actix::Message for ListRooms {
    type Result = Vec<String>;
}

/// Establishes session, always the first message sent by the client. Contains
/// the session ID acquired on login. Without it a session can't be established.
#[derive(Message, Debug)]
#[rtype(result = "String")]
pub struct Session {
    /// The session ID that gets passed to the server
    pub session_id: String,
    /// The address of the server
    pub address: Recipient<Message>,
}

pub struct Users;
impl actix::Message for Users {
    type Result = Vec<String>;
}

/// Join room, if room does not exists create new one.
///
/// `id`: the client session ID,
/// `room_name`: the room name
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    pub id: String,
    pub room_name: String,
}
