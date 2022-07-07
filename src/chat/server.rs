//! `ChatServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ChatServer`.
use crate::chat::ez_handler;
use crate::chat::models::{
    ChatMessage, ClientMessage, Connect, Disconnect, Join, LeL, ListRooms, LoL, Message,
    MessageData, Session, Users,
};
use crate::models::user::ChatUser;
use actix::prelude::*;
use colored::Colorize;
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    //  time::{Instant, Duration}
};

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

/// `ChatServer` manages chat rooms and responsible for coordinating chat sessions.
/// 
/// It is the actor responsible for keeping track of which session ID points to which
/// actor address and which sessions are communicating with one another. It also keeps track
/// of currently connected users and processes messages from other actors.
/// 
/// Since it is also an actor, it can send messages to other actors, namely the
/// session actors which then finally send the message to the client.
#[derive(Debug)]
pub struct ChatServer {
    /// Sessions map a session ID with its actor address
    sessions: HashMap<String, Recipient<Message>>,
    /// Rooms map a session ID to other session IDs
    rooms: HashMap<String, HashSet<String>>,
    /// The total connected users
    users_connected: Vec<ChatUser>,
}

impl ChatServer {
    pub fn new() -> ChatServer {
        ChatServer {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
            users_connected: vec![],
        }
    }
}

pub fn reg() {
   // ez_register!(LoL, test_macro, ());
    ez_register!(LeL, test_macro2, String);
    ez_register_block!(LoL, {for _ in 0..10 {println!("LO")}});
}

fn test_macro<M>(msg: M)
where
    M: actix::Message,
{
    for _ in 0..5 {
        print!("LO-");
    }
}
fn test_macro2<M>(msg: M) -> String
where
    M: actix::Message,
{
    "Helloworld".to_string()
}

impl ChatServer {
    /// Send a message to a specific actor.
    fn send_message(&self, receiver: &str, message: String) {
        if let Some(sessions) = self.rooms.get(receiver) {
            for id in sessions {
                if let Some(address) = self.sessions.get(id) {
                    let _ = address.do_send(Message(message.clone()));
                }
            }
        }
    }
    /// Send message to all actors
    fn send_global(&self, message: String) {
        println!("SENDING GLOBAL : {:?}", message);
        for address in self.sessions.values() {
            println!("TO : {:?}", address);
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

/// Message received upon connection with client. Adds the newly connected user to the
/// connected_users vec if they are new, otherwise sets their status to connected
impl Handler<Connect> for ChatServer {
    type Result = ();
    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        print!("{}", "USER CONNECTED : ".green());
        println!("{:?}", msg.user);

        self.send_global(
            ez_handler::generate_data_message(
                MessageData::List(vec![msg.user.clone()]),
                "user_connected",
            )
            .unwrap(),
        );

        // Cycle through the users to see if they were previously connected
        for (i, user) in self.users_connected.clone().iter().enumerate() {
            if user.id == msg.user.id {
                // If they were just set their status to connected and return
                self.users_connected[i].connected = true;
                return;
            }
        }
        // Otherwise push them to the state
        self.users_connected.push(msg.user);
    }
}

/// Message received when an actor gets dropped. Sets users' connected status to false.
impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        print!("{}", "USER DISCONNECTED : ".red());
        println!("{:?}", msg);

        // Remove the user from the registered sessions
        if self.sessions.remove(&msg.session_id).is_some() {
            // Remove the session from all rooms
            for (id, sessions) in &mut self.rooms {
                sessions.remove(id);
            }
        }
        self.clean_rooms();

        for (i, user) in self.users_connected.clone().iter().enumerate() {
            if user.id == msg.session_id {
                if let Some(user) = self.users_connected.get_mut(i) {
                    user.connected = false;
                }
            }
        }
    }
}

/// Gets called when connection is established, returns the session ID after
/// registering the session and joining its own room.
impl Handler<Session> for ChatServer {
    type Result = String;

    fn handle(&mut self, msg: Session, _: &mut Context<Self>) -> Self::Result {
        print!("{}", "GOT SESSION MESSAGE : ".cyan());
        println!("{:?}", &msg);

        let id = msg.session_id;

        self.sessions.insert(id.clone(), msg.address);
        self.rooms
            .entry(id.to_owned())
            .or_insert_with(HashSet::new)
            .insert(id.clone());

        println!("SESSIONS : {:?}", self.sessions);
        println!("ROOMS : {:?}", self.rooms);

        id
    }
}

/// Sends a list of all the users with established sessions with the server
impl Handler<Users> for ChatServer {
    type Result = Vec<ChatUser>;
    fn handle(&mut self, _: Users, _: &mut Context<Self>) -> Self::Result {
        print!("{}", "SENDING USERS".cyan());
        self.users_connected.clone()
    }
}

/// Handler for the actual
impl<T: Serialize> Handler<ClientMessage<T>> for ChatServer {
    type Result = MessageResult<ClientMessage<T>>;

    fn handle(&mut self, msg: ClientMessage<T>, _: &mut Context<Self>) -> Self::Result {
        if let Some(ref body) = msg.body {
            let chat_message: ChatMessage = match serde_json::from_str(&body) {
                Ok(msg) => msg,
                Err(e) => {
                    println!("E: {:?}", e);
                    ChatMessage {
                        id: String::new(),
                        sender_id: String::new(),
                        receiver_id: String::new(),
                        content: String::new(),
                        read: false,
                    }
                }
            };
            println!("MSG: {:?}", &msg.header);

            if let Some(rooms) = self.rooms.get(&chat_message.sender_id) {
                if let Some(receiver) = rooms.get(&chat_message.receiver_id) {
                    println!("SENDING MSG TO : {:?}", self.sessions.get(receiver));
                    //self.send_message(receiver, &body);
                }
            }
        }
        MessageResult(())
    }
}

/// Handler for `ListRooms` message.
impl Handler<ListRooms> for ChatServer {
    type Result = MessageResult<ListRooms>;

    fn handle(&mut self, _: ListRooms, _: &mut Context<Self>) -> Self::Result {
        let mut rooms = Vec::new();

        for key in self.rooms.keys() {
            rooms.push(key.to_owned())
        }

        MessageResult(rooms)
    }
}

/// Join room, send disconnect message to old room
/// send join message to new room
impl Handler<Join> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let id = msg.id;
        let name = msg.room_name;
        let mut rooms = Vec::new();

        // remove session from all rooms
        for (room, sessions) in &mut self.rooms {
            if sessions.remove(&id) {
                rooms.push(room.to_owned());
            }
        }
        // send message to other users
        for room in rooms {
            // self.send_message(&room, "Someone disconnected");
        }

        self.rooms
            .entry(name.clone())
            .or_insert_with(HashSet::new)
            .insert(id.clone());

        // self.send_message(&name, "Someone connected");
    }
}
