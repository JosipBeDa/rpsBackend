use crate::{
    actors::chat::models::{messages::ChatMessage, room::PublicRoom},
};
use actix::Message;
use serde::{Deserialize, Serialize};

#[derive(Message, Debug, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct StoreHoFEntry {
    pub user_id: String,
}

#[derive(Message, Debug, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct StoreRoom {
    pub room: PublicRoom,
    pub admin_id: String,
}

#[derive(Message, Debug, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct StoreChatMessage {
    pub message: ChatMessage,
    pub is_room: bool,
}

#[derive(Message, Debug, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct StoreRoomConnection {
    pub room_id: String,
    pub user_id: String,
}
