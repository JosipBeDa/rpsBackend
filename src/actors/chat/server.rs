//! `ChatServer` is an actor. It maintains the state of connected client sessions
//! and with whom the sessions are communicating.
use super::models::{
    chat_user::ChatUser,
    messages::{ChatMessage, CreateRoom, Join, Read},
    room::{PublicRoom, RoomData},
};
use crate::actors::{
    db::{
        manager::DBManager,
        messages::{StoreChatMessage, StoreRoom, StoreRoomConnection},
    },
    ez_handler,
    models::messages::{
        client_message::{ClientMessage, MessageData, SocketMessage},
        connection::{Connect, Disconnect},
    },
};
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
pub struct ChatServer {
    /// Sessions map a session ID with its actor address
    sessions: HashMap<String, Recipient<SocketMessage>>,
    /// Maps session IDs to other session IDs
    id_pointers: HashMap<String, String>,
    public_rooms: HashMap<String, PublicRoom>,
    /// Messages
    messages: Vec<ChatMessage>,
    /// The total connected users
    users: HashMap<String, ChatUser>,
    /// The database connection
    db_manager: Addr<DBManager>,
}

impl ChatServer {
    pub fn new(db_manager: Addr<DBManager>) -> Self {
        Self {
            sessions: HashMap::new(),
            id_pointers: HashMap::new(),
            public_rooms: HashMap::new(),
            users: HashMap::new(),
            messages: vec![],
            db_manager,
        }
    }
    /// Send a message to whoever the sender is pointing to
    fn send(&self, sender: &str, message: String) {
        if let Some(id) = self.id_pointers.get(sender) {
            if let Some(address) = self.sessions.get(id) {
                let _ = address.do_send(SocketMessage(message.clone()));
            }
        }
    }
    /// Directly send a message to the address of the given session ID.
    fn send_direct(&self, receiver: &str, message: String) {
        if let Some(address) = self.sessions.get(receiver) {
            let _ = address.do_send(SocketMessage(message.clone()));
        }
    }
    /// Send a message to all actors
    fn broadcast(&self, message: String) {
        info!("{}{:?}", "BROADCASTING : ".blue(), message);
        for address in self.sessions.values() {
            address.do_send(SocketMessage(message.clone()));
        }
    }

    /// Send a message to all users in a specific room
    fn _room_broadcast(&self, room_id: &str, message: String) {
        if let Some(room) = self.public_rooms.get(room_id) {
            for user_id in room.get_user_ids() {
                if let Some(address) = self.sessions.get(&user_id) {
                    address.do_send(SocketMessage(message.clone()))
                }
            }
        }
    }

    /// Gets all messages sent and received between the two given ids. If they are private messages and
    /// `id` is the receiver of those messages, read them.
    fn get_messages(&mut self, id: &str, other_id: &str) -> Vec<ChatMessage> {
        let mut messages = vec![];
        for message in &mut self.messages {
            // If the user is the receiver read the message
            if message.receiver_id == id && message.sender_id == other_id {
                if message.read == false {
                    message.read = true;
                }
                messages.push(message.clone());
                continue;
            }
            // Otherwise just prepare it for sending
            if message.receiver_id == other_id && message.sender_id == id {
                messages.push(message.clone());
            }
        }
        messages
    }

    /// Returns all registered public rooms in a vec
    fn get_rooms(&self) -> Vec<PublicRoom> {
        self.public_rooms.values().cloned().collect()
    }

    /// Clean empty id_pointers
    fn clean_rooms(&mut self) {
        for room in self.id_pointers.clone().into_keys() {
            if let Some(sessions) = self.id_pointers.get(&room) {
                if sessions.is_empty() {
                    self.id_pointers.remove(&room);
                }
            }
        }
    }
}

/// Make actor from `ChatServer`
impl Actor for ChatServer {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Context<Self>) {
        info!("{}", "Started Chat Server".green());
    }
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
        self.id_pointers
            .entry(id.to_owned())
            .or_insert_with(|| id.clone());

        // Send session string to self
        self.send_direct(
            &id,
            ez_handler::generate_message::<String>("session", MessageData::String(id.clone()))
                .unwrap(),
        );

        // Send all users to self
        self.send_direct(
            &id,
            ez_handler::generate_message(
                "users",
                MessageData::List(self.users.clone().into_values().collect()),
            )
            .unwrap(),
        );

        // Send all public rooms to self
        if self.public_rooms.len() > 0 {
            self.send_direct(
                &id,
                ez_handler::generate_message::<RoomData>(
                    "room",
                    MessageData::Room(RoomData::Rooms(self.get_rooms())),
                )
                .unwrap(),
            );
        }
    }
}

/// Message received when an actor gets dropped. Sets users' connected status to false.
/// Sends a global message with the disconnecting user's ID and removes their room entry.
impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        info!("{}{:?}", "USER DISCONNECTED : ".red(), msg);

        if self.sessions.remove(&msg.session_id).is_some() {
            self.id_pointers.remove(&msg.session_id);
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
            // Push it to the in memory store
            self.messages.push(msg.clone());

            let message = ez_handler::generate_message::<ChatMessage>(
                "chat_message",
                MessageData::ChatMessage(msg.clone()),
            )
            .unwrap();

            // Store it and return if it's intended for a room
            if let Some(room) = self.public_rooms.get_mut(&msg.receiver_id) {
                room.store_message(msg.clone());
            }
            if let Some(room) = self.public_rooms.get(&msg.receiver_id) {
                for user_id in self.users.keys() {
                    if let Some(receiver) = self.id_pointers.get(user_id) {
                        if receiver.eq(&room.id) {
                            self.send_direct(user_id, message.clone())
                        }
                    }
                }
                self.db_manager.do_send(StoreChatMessage {
                    message: msg,
                    is_room: true,
                });
                return;
            } else {
                self.db_manager.do_send(StoreChatMessage {
                    message: msg.clone(),
                    is_room: false,
                });
                // Send it only if it's not being sent to self
                if msg.receiver_id != msg.sender_id {
                    self.send(&msg.sender_id, message.clone());
                }
                self.send_direct(&&msg.sender_id, message);
            }
        }
    }
}

impl Handler<Join> for ChatServer {
    type Result = Vec<ChatMessage>;

    fn handle(&mut self, message: Join, _: &mut Context<Self>) -> Self::Result {
        let Join { id, room_id } = message;
        info!("{}{}{}{}", "JOINING : ".cyan(), id, " => ".cyan(), room_id);

        // Set the sender to point to the receiver
        self.id_pointers.remove(&id);
        self.id_pointers
            .entry(id.clone())
            .or_insert_with(|| room_id.clone());

        // If the user is joining a room, broadcast it to everyone in the room if they're new
        if let Some(public_room) = self.public_rooms.get_mut(&room_id) {
            if !public_room.has_user(&id) {
                public_room.set_user(&id);
                self.broadcast(
                    ez_handler::generate_message::<RoomData>(
                        "room",
                        MessageData::Room(RoomData::Joined((id.clone(), room_id.clone()))),
                    )
                    .unwrap(),
                );
            }
        }

        // If joining a public room return its messages
        if let Some(public_room) = self.public_rooms.get(&room_id) {
            self.db_manager.do_send(StoreRoomConnection {
                room_id: public_room.id.clone(),
                user_id: id,
            });
            return public_room.get_messages();
        }

        // Get all associated messages
        let messages = self.get_messages(&id, &room_id);

        // Messages sent to self are automatically read
        if id != room_id && messages.len() > 0 {
            self.send(
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
        self.send_direct(
            sender.as_ref(),
            ez_handler::generate_message("read", MessageData::List(message.messages)).unwrap(),
        );
    }
}

/// Creates and stores a public room then broadcasts it to everyone connected
impl Handler<CreateRoom> for ChatServer {
    type Result = ();
    fn handle(&mut self, message: CreateRoom, _: &mut Context<Self>) -> Self::Result {
        info!("{}{:?}", "CREATING ROOM WITH : ".cyan(), message.sender_id);
        let id = uuid::Uuid::new_v4().to_string();
        let room = PublicRoom::new_insert(&id, &message.sender_id, &message.name);
        self.public_rooms.insert(id.clone(), room.clone());
        self.db_manager.do_send(StoreRoom {
            room: room.clone(),
            admin_id: message.sender_id,
        });
        self.broadcast(
            ez_handler::generate_message::<RoomData>(
                "room",
                MessageData::Room(RoomData::Room(room)),
            )
            .unwrap(),
        );
    }
}
