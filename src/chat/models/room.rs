use super::messages::ChatMessage;
use actix::MessageResponse;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(MessageResponse, Debug, Serialize, Deserialize, Clone)]
pub struct PublicRoom {
    users: HashSet<String>,
    messages: Vec<ChatMessage>,
}

impl PublicRoom {
    pub fn new_insert(id: &str) -> Self {
        let mut room = Self {
            users: HashSet::new(),
            messages: vec![],
        };
        let _ = room.users.insert(id.to_string());
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

    pub fn store_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
    }
}
