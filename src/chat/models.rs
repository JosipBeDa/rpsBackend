//! Contains the message models
use crate::models::user::ChatUser;
use actix::prelude::*;
use serde::{Deserialize, Serialize};

use crate::rps::models::RPSData;

/// Chat server sends these messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

/// Creates a new chat session. Always the first message sent from the client.
/// Contains the details of the connecting user.
#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct Connect {
    pub user: ChatUser,
    pub address: Recipient<Message>,
}

/// When the server
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Disconnect {
    /// The ID of the session to disconnect
    pub session_id: String,
}

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
    RPS(RPSData),
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

/// The expected struct to usewhen sending and receiving chat messages. This is the actual
/// message that gets stored in the database.
#[derive(Message, Debug, Serialize, Deserialize, Clone)]
#[rtype(result = "()")]
pub struct ChatMessage {
    /// Each chat message has a specific id.
    pub id: String,
    /// The session ID of the sender. Note that every session ID corresponds
    /// to a user in the database because they cannot use the chat unless they are logged in.
    pub sender_id: String,
    /// The session ID of the receiver.
    pub receiver_id: String,
    /// The text content of the message.
    pub content: String,
    /// Flag indicating whether the receiver has read the message. If it is a public message
    /// (i.e message sent to rooms with multiple receivers) this flag is omitted.
    pub read: bool,
}

/// Maps `id` to `room_id` in `ChatServer`'s rooms. Also reads messages.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Join {
    pub id: String,
    pub room_id: String,
}
/// Returns a vec of read message IDs where `id` was the receiver and
/// `room_id` was the sender.
impl actix::Message for Join {
    type Result = Vec<String>;
}
/// Contains a vec of read messages.
#[derive(Message, Debug, Serialize, Deserialize, Clone)]
#[rtype(result = "()")]
pub struct Read {
    pub messages: Vec<ChatMessage>,
}
