use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Eq, Hash)]
pub struct ChatUser {
    pub id: String,
    pub username: String,
    pub connected: bool,
}

impl ToString for ChatUser {
    fn to_string(&self) -> String {
        serde_json::to_string(self).expect("Couldn't serialize user")
    }
}
