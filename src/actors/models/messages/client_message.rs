use actix::Message;
use serde::{Deserialize, Serialize};

use crate::actors::{
    chat::models::{
        chat_user::ChatUser,
        messages::{ChatMessage, CreateRoom, Join},
        room::RoomData,
    },
    rps::models::RPSData,
};

/// Chat server sends these messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct SocketMessage(pub String);

/// The main message format ez_socket expects.
#[derive(Message, Debug, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct ClientMessage<T: Serialize> {
    /// The header registered in the ez_handler. Indicates the type of the message.
    pub header: String,
    /// Contains the message data.
    pub data: MessageData<T>,
}

/// Represents the type of message data.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum MessageData<T>
where
    T: Serialize,
{
    String(String),
    List(Vec<T>),
    User(ChatUser),
    ChatMessage(ChatMessage),
    Join(Join),
    /// Contains all data related to RPS games.
    RPS(RPSData),
    /// Contains all data related to rooms.
    Room(RoomData),
    CreateRoom(CreateRoom),
}

/// Shortcuts for serializing messages to JSON.
impl<T> ToString for ClientMessage<T>
where
    T: Serialize,
{
    fn to_string(&self) -> String {
        serde_json::to_string(self).expect("Couldn't serialize struct")
    }
}
impl ToString for ChatMessage {
    fn to_string(&self) -> String {
        serde_json::to_string(self).expect("Couldn't serialize struct")
    }
}
