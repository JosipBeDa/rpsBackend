//! `ChatServer` actor. It maintains a list of connected client sessions
//! and with whom the sessions are communicating.
use crate::chat::ez_handler;
use crate::chat::models::{ChatMessage, ClientMessage, Connect, Disconnect, Message, MessageData};
use crate::models::user::ChatUser;
use actix::prelude::*;
use colored::Colorize;
use serde::Serialize;
use std::collections::HashMap;
use tracing::info;

use super::models::{Join, Read};
/// `ChatServer` is an actor that manages chat rooms and is responsible for coordinating chat sessions.
///
/// It is the actor responsible for keeping track of which session ID points to which
/// actor address (`sessions`) and which sessions are communicating with one another (`rooms`).
/// It also keeps track of currently connected users and processes messages from other actors.
#[derive(Debug)]
pub struct ChatServer {
    /// Sessions map a session ID with its actor address
    sessions: HashMap<String, Recipient<Message>>,
    /// Maps session IDs to other session IDs
    rooms: HashMap<String, String>,
    /// Messages
    messages: Vec<ChatMessage>,
    /// The total connected users
    users: HashMap<String, ChatUser>,
}

impl ChatServer {
    pub fn new() -> ChatServer {
        ChatServer {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
            users: HashMap::new(),
            messages: vec![],
        }
    }
    /// Send a message to a specific room. The `sender` is used to fetch all IDs
    fn send_message(&self, sender: &str, message: String) {
        if let Some(id) = self.rooms.get(sender) {
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
    fn send_global(&self, message: String) {
        info!("{}{:?}", "SENDING GLOBAL : ".blue(), message);
        for address in self.sessions.values() {
            address.do_send(Message(message.clone()));
        }
    }

    /// Clean empty rooms
    fn clean_rooms(&mut self) {
        for room in self.rooms.clone().into_keys() {
            if let Some(sessions) = self.rooms.get(&room) {
                if sessions.is_empty() {
                    self.rooms.remove(&room);
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
        self.send_global(
            ez_handler::generate_message::<ChatUser>(
                "user_connected",
                MessageData::User(msg.user.clone()),
            )
            .unwrap(),
        );

        let id = msg.user.id.clone();
        self.sessions.insert(id.clone(), msg.address);
        self.rooms
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
            ez_handler::generate_message("messages", MessageData::List(self.messages.clone()))
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
            self.rooms.remove(&msg.session_id);
        }

        if let Some(user) = self.users.get_mut(&msg.session_id) {
            user.connected = false;
        }

        self.send_global(
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
    type Result = Vec<String>;

    fn handle(&mut self, message: Join, _: &mut Context<Self>) -> Self::Result {
        let Join { id, room_id } = message;
        info!("{}{}{}{}", "JOINING : ".cyan(), id, " => ".cyan(), room_id);

        self.rooms.remove(&id);
        self.rooms
            .entry(id.clone())
            .or_insert_with(|| room_id.clone());

        info!(
            "{}{:?}{}{:?}",
            "ROOMS : ".cyan(),
            self.rooms,
            " SESSIONS ".cyan(),
            self.sessions.keys()
        );

        let mut read = vec![];

        for msg in &mut self.messages {
            if msg.receiver_id == id && msg.sender_id == room_id && msg.read == false {
                msg.read = true;
                read.push(msg.id.clone());
            }
        }

        // Messages sent to self are automatically read
        if id != room_id && read.len() > 0 {
            self.send_message(
                &id,
                ez_handler::generate_message("read", MessageData::List(read.clone())).unwrap(),
            );
        }
        read
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
