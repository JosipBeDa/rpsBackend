//! Contains the message models
use actix::Message;
use serde::{Deserialize, Serialize};

/// The expected struct to use when sending and receiving chat messages. This is the actual
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

/// Maps `id` to `room_id` in `ChatServer`'s rooms. Also reads messages. Returns all messages
/// concerning the joining party and the party being joined
#[derive(Message, Debug, Serialize, Deserialize, Clone)]
#[rtype(result = "Vec<ChatMessage>")]
pub struct Join {
    pub id: String,
    pub room_id: String,
}
/// Contains a vec of read messages.
#[derive(Message, Debug, Serialize, Deserialize, Clone)]
#[rtype(result = "()")]
pub struct Read {
    pub messages: Vec<ChatMessage>,
}

/// Creates a public chat room
#[derive(Message, Debug, Serialize, Deserialize, Clone)]
#[rtype(result = "()")]
pub struct CreateRoom {
    pub sender_id: String,
    pub name: String,
}
