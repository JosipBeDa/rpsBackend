use super::messages::ChatMessage;
use actix::MessageResponse;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(MessageResponse, Debug, Serialize, Deserialize, Clone)]
pub struct PublicRoom {
    pub id: String,
    pub name: String,
    pub users: HashSet<String>,
    pub messages: Vec<ChatMessage>,
}

impl PublicRoom {
    pub fn new_insert(id: &str, user_id: &str, name: &str) -> Self {
        let mut room = Self {
            id: id.to_string(),
            name: name.to_string(),
            users: HashSet::new(),
            messages: vec![],
        };
        let _ = room.users.insert(user_id.to_string());
        room
    }

    pub fn get_messages(&self) -> Vec<ChatMessage> {
        self.messages.clone()
    }

    pub fn get_user_ids(&self) -> HashSet<String> {
        self.users.clone()
    }

    pub fn set_user(&mut self, id: &str) {
        self.users.insert(id.to_string());
    }

    pub fn remove_user(&mut self, id: &str) {
        self.users.remove(id);
    }

    pub fn has_user(&self, id: &str) -> bool {
        if self.users.contains(id) {
            return true;
        }
        false
    }

    pub fn store_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RoomData {
    Message(ChatMessage),
    Room(PublicRoom),
    Rooms(Vec<PublicRoom>),
    Joined((String, String)),
}
