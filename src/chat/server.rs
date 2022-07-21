//! `ChatServer` actor. It maintains a list of connected client sessions
//! and with whom the sessions are communicating.
use super::models::messages::{
    ChatMessage, ClientMessage, Connect, Disconnect, Join, Message, MessageData, Read, CreateRoom,
};
use super::models::room::PublicRoom;
use crate::chat::ez_handler;
use crate::models::user::ChatUser;
use crate::state::db_pool;
use actix::prelude::*;
use colored::Colorize;
use serde::Serialize;
use std::collections::HashMap;
use tracing::info;
/// `ChatServer` is an actor that manages chat rooms and is responsible for coordinating chat sessions.
///
/// It is the actor responsible for keeping track of which session ID points to which
/// actor address (`sessions`) and which sessions are communicating with one another (`rooms`).
/// It also keeps track of currently connected users and processes messages from other actors.
//#[derive(Debug)]
pub struct ChatServer {
    /// Sessions map a session ID with its actor address
    sessions: HashMap<String, Recipient<Message>>,
    /// Maps session IDs to other session IDs
    private_rooms: HashMap<String, String>,
    public_rooms: HashMap<String, PublicRoom>,
    /// Messages
    messages: Vec<ChatMessage>,
    /// The total connected users
    users: HashMap<String, ChatUser>,
    /// The database connection
    db_pool: db_pool::PgPool,
}

impl ChatServer {
    pub fn new(db_pool: db_pool::PgPool) -> ChatServer {
        ChatServer {
            sessions: HashMap::new(),
            private_rooms: HashMap::new(),
            public_rooms: HashMap::new(),
            users: HashMap::new(),
            messages: vec![],
            db_pool,
        }
    }
    /// Send a message to a specific room. The `sender` is used to fetch all IDs it's pointing to
    fn send_message(&self, sender: &str, message: String) {
        if let Some(id) = self.private_rooms.get(sender) {
            info!(
                "{}{:?}{}{:?}",
                "SENDING MESSAGE : ".blue(),
                message,
                " TO ".blue(),
                id
            );

            if let Some(address) = self.sessions.get(id) {
                let _ = address.do_send(Message(message.clone()));
            }
        }
    }
    /// Directly sends a message to the address of the given session ID.
    fn send_data(&self, receiver: &str, message: String) {
        if let Some(address) = self.sessions.get(receiver) {
            let _ = address.do_send(Message(message.clone()));
        }
    }
    /// Send message to all actors
    fn broadcast(&self, message: String) {
        info!("{}{:?}", "SENDING GLOBAL : ".blue(), message);
        for address in self.sessions.values() {
            address.do_send(Message(message.clone()));
        }
    }

    /// Gets all messages sent to and received from the given user
    fn get_private_messages(&self, user_id: &str) -> Vec<ChatMessage> {
        let mut messages = vec![];
        for message in &self.messages {
            if message.sender_id == user_id || message.receiver_id == user_id {
                messages.push(message.clone());
            }
        }
        messages
    }

    /// Clean empty private_rooms
    fn clean_rooms(&mut self) {
        for room in self.private_rooms.clone().into_keys() {
            if let Some(sessions) = self.private_rooms.get(&room) {
                if sessions.is_empty() {
                    self.private_rooms.remove(&room);
                }
            }
        }
    }
}

/// Make actor from `ChatServer`
impl Actor for ChatServer {
    type Context = Context<Self>;
}

/// Message received upon connection with client. Registers the user if they are new,
/// otherwise sets their status to connected. Sends a global message with the connecting user's data and
/// sends the following to the connecting user:
/// - session
/// - users
/// - messages
impl Handler<Connect> for ChatServer {
    type Result = ();
    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        info!("{}{:?}", "USER CONNECTED : ".green(), msg.user);

        if let Some(user) = self.users.get_mut(&msg.user.id) {
            user.connected = true;
        } else {
            self.users.insert(msg.user.id.clone(), msg.user.clone());
        }

        // Notify all users
        self.broadcast(
            ez_handler::generate_message::<ChatUser>(
                "user_connected",
                MessageData::User(msg.user.clone()),
            )
            .unwrap(),
        );

        // Insert into session
        let id = msg.user.id.clone();
        self.sessions.insert(id.clone(), msg.address);
        self.private_rooms
            .entry(id.to_owned())
            .or_insert_with(|| id.clone());

        // Send session string to self
        self.send_data(
            &id,
            ez_handler::generate_message::<String>("session", MessageData::String(id.clone()))
                .unwrap(),
        );

        // Send all users to self
        self.send_data(
            &id,
            ez_handler::generate_message(
                "users",
                MessageData::List(self.users.clone().into_values().collect()),
            )
            .unwrap(),
        );

        // Send all messages to self
        self.send_data(
            &id,
            ez_handler::generate_message(
                "messages",
                MessageData::List(self.get_private_messages(&id)),
            )
            .unwrap(),
        );
    }
}

/// Message received when an actor gets dropped. Sets users' connected status to false.
/// Sends a global message with the disconnecting user's ID and removes their room entry.
impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        info!("{}{:?}", "USER DISCONNECTED : ".red(), msg);

        if self.sessions.remove(&msg.session_id).is_some() {
            self.private_rooms.remove(&msg.session_id);
        }

        if let Some(user) = self.users.get_mut(&msg.session_id) {
            user.connected = false;
        }

        self.broadcast(
            ez_handler::generate_message::<String>(
                "user_disconnected",
                MessageData::String(msg.session_id.clone()),
            )
            .unwrap(),
        );

        self.clean_rooms();
    }
}

/// Handler for the chat message. Stores the received message and sends it
/// to the receiver.
impl<T: Serialize> Handler<ClientMessage<T>> for ChatServer {
    type Result = ();

    fn handle(&mut self, message: ClientMessage<T>, _: &mut Context<Self>) -> Self::Result {
        if let MessageData::ChatMessage(msg) = message.data {
            self.messages.push(msg.clone());

            let message = ez_handler::generate_message::<ChatMessage>(
                "chat_message",
                MessageData::ChatMessage(msg.clone()),
            )
            .unwrap();

            // Send it only if it's not being sent to self
            if msg.receiver_id != msg.sender_id {
                self.send_message(&msg.sender_id, message.clone());
            }
            self.send_data(&&msg.sender_id, message);
        }
    }
}

impl Handler<Join> for ChatServer {
    type Result = Vec<ChatMessage>;

    fn handle(&mut self, message: Join, _: &mut Context<Self>) -> Self::Result {
        let Join { id, room_id } = message;
        info!("{}{}{}{}", "JOINING : ".cyan(), id, " => ".cyan(), room_id);

        // If joining a public room simply return its messages
        if let Some(public_room) = self.public_rooms.get_mut(&room_id) {
            public_room.set_user(&id);
            return public_room.get_messages();
        }

        // Set the sender to point to the receiver
        self.private_rooms.remove(&id);
        self.private_rooms
            .entry(id.clone())
            .or_insert_with(|| room_id.clone());

        // Get all associated messages
        let mut messages = vec![];
        for msg in &mut self.messages {
            if msg.receiver_id == id && msg.sender_id == room_id && msg.read == false {
                if msg.read == false {
                    msg.read = true;
                }
                messages.push(msg.clone());
            }
        }

        // Messages sent to self are automatically read
        if id != room_id && messages.len() > 0 {
            self.send_message(
                &id,
                ez_handler::generate_message("read", MessageData::List(messages.clone())).unwrap(),
            );
        }
        messages
    }
}

impl Handler<Read> for ChatServer {
    type Result = ();
    fn handle(&mut self, message: Read, _: &mut Context<Self>) -> Self::Result {
        info!("{}{:?}", "READING MESSAGES : ".cyan(), message.messages);
        let mut sender = String::new();
        for message in message.messages.clone() {
            let ChatMessage { id, sender_id, .. } = message;
            sender = sender_id;
            for i in 0..self.messages.len() {
                if id == self.messages[i].id {
                    self.messages[i].read = true;
                    break;
                }
            }
        }
        self.send_data(
            sender.as_ref(),
            ez_handler::generate_message("read", MessageData::List(message.messages)).unwrap(),
        );
    }
}

impl Handler<CreateRoom> for ChatServer {
    type Result = ();
    fn handle(&mut self, message: CreateRoom, _: &mut Context<Self>) -> Self::Result {
        info!("{}{:?}", "CREATING ROOM WITH : ".cyan(), message.sender_id);
        let id = uuid::Uuid::new_v4().to_string();
        let room = PublicRoom::new_insert(&message.sender_id);
        self.public_rooms.insert(id.clone(), room.clone());
        self.broadcast(ez_handler::generate_message::<CreateRoom>("room", MessageData::Room((id, room))).unwrap());
    }
}
