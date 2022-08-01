use actix::{Message, Recipient};

use crate::actors::chat::models::chat_user::ChatUser;

use super::client_message::SocketMessage;

/// Registers a session with the receiving actor.
#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct Connect {
    pub user: ChatUser,
    pub address: Recipient<SocketMessage>,
}

/// Removes the corresponding session from the actor's session store
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Disconnect {
    /// The ID of the session to disconnect
    pub session_id: String,
}
