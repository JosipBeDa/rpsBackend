//! `ChatServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ChatServer`.
use crate::chat::ez_handler;
use crate::chat::models::{ChatMessage, ClientMessage, Connect, Disconnect, Message, MessageData};
use crate::models::user::ChatUser;
use actix::prelude::*;
use colored::Colorize;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use tracing::info;

/// Shortcut for implementing the `actix::Handler` trait for any given struct that implements the
/// `actix::Message` trait. Used when we have to access the contents of the message.
#[macro_export]
macro_rules! ez_register {
    ($msg: ty,$func: ident, $ret: ty) => {
        impl actix::Handler<$msg> for crate::chat::server::ChatServer {
            type Result = $ret;
            fn handle(&mut self, msg: $msg, _: &mut Context<Self>) -> Self::Result {
                $func(msg)
            }
        }
    };
}
/// Shortcut for implementing the `actix::Handler` trait for any given struct that implements the
/// `actix::Message` trait. Used when we have don't need the message contents, cheaper because we don't
/// create an extra function pointer.
#[macro_export]
macro_rules! ez_register_block {
    ($msg: ty, $blck: block) => {
        impl actix::Handler<$msg> for crate::chat::server::ChatServer {
            type Result = ();
            fn handle(&mut self, _: $msg, _: &mut Context<Self>) -> Self::Result $blck
        }
    };
}

/// `ChatServer` manages chat rooms and is responsible for coordinating chat sessions.
///
/// It is the actor responsible for keeping track of which session ID points to which
/// actor address (`sessions`) and which sessions are communicating with one another (`rooms`).
/// It also keeps track of currently connected users and processes messages from other actors.
#[derive(Debug)]
pub struct ChatServer {
    /// Sessions map a session ID with its actor address
    sessions: HashMap<String, Recipient<Message>>,
    /// Rooms map a session ID to other session IDs
    rooms: HashMap<String, HashSet<String>>,
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
            messages: vec![],
            users: HashMap::new(),
        }
    }
    /// Send a message to a specific room. The `sender` is used to fetch all IDs
    fn send_message(&self, sender: &str, message: String) {
        if let Some(sessions) = self.rooms.get(sender) {
            info!(
                "{}{:?}{}{:?}",
                "SENDING MESSAGE : ".blue(),
                message,
                " TO ".blue(),
                sessions
            );
            for id in sessions {
                if let Some(address) = self.sessions.get(id) {
                    let _ = address.do_send(Message(message.clone()));
                }
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
                MessageData::User(msg.user.clone()),
                "user_connected",
            )
            .unwrap(),
        );

        let id = msg.user.id.clone();

        self.sessions.insert(id.clone(), msg.address);
        self.rooms
            .entry(id.to_owned())
            .or_insert_with(HashSet::new)
            .insert(id.clone());

        info!("{}{:?}", "SESSIONS : ".blue(), self.sessions);
        info!("{}{:?}", "ROOMS : ".blue(), self.rooms);

        // Send session string to self
        self.send_data(
            &id,
            ez_handler::generate_message::<String>(MessageData::String(id.clone()), "session")
                .unwrap(),
        );

        // Send all users to self
        self.send_data(
            &id,
            ez_handler::generate_message(
                MessageData::List(self.users.clone().into_values().collect()),
                "users",
            )
            .unwrap(),
        );

        // Send all messages to self
        self.send_data(
            &id,
            ez_handler::generate_message(MessageData::List(self.messages.clone()), "messages")
                .unwrap(),
        );
    }
}

/// Message received when an actor gets dropped. Sets users' connected status to false.
/// Sends a global message with the disconnecting user's ID and removes their session ID from all rooms.
impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        info!("{}{:?}", "USER DISCONNECTED : ".red(), msg);

        if self.sessions.remove(&msg.session_id).is_some() {
            for (id, sessions) in &mut self.rooms {
                sessions.remove(id);
            }
        }

        if let Some(user) = self.users.get_mut(&msg.session_id) {
            user.connected = false;
        }

        self.send_global(
            ez_handler::generate_message::<String>(
                MessageData::String(msg.session_id.clone()),
                "user_disconnected",
            )
            .unwrap(),
        );

        self.clean_rooms();
    }
}

/// Handler for the chat message.
impl<T: Serialize> Handler<ClientMessage<T>> for ChatServer {
    type Result = ();

    fn handle(&mut self, message: ClientMessage<T>, _: &mut Context<Self>) -> Self::Result {
        // Check if the message is a chat message
        if let MessageData::ChatMessage(msg) = message.data {
            // Save the message
            self.messages.push(msg.clone());

            let message = ez_handler::generate_message::<ChatMessage>(
                MessageData::ChatMessage(msg.clone()),
                "chat_message",
            )
            .unwrap();
            self.send_message(&msg.sender_id, message.clone());
        }
    }
}
