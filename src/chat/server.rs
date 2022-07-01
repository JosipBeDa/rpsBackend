//! `ChatServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ChatServer`.
use crate::chat::models::{
    ChatMessage, ClientMessage, Connect, Disconnect, Join, ListRooms, Message, Session, Users,
};
use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    //  time::{Instant, Duration}
};

/// `ChatServer` manages chat rooms and responsible for coordinating chat session.
#[derive(Debug)]
pub struct ChatServer {
    sessions: HashMap<String, Recipient<Message>>,
    rooms: HashMap<String, HashSet<String>>,
    visitor_count: Arc<AtomicUsize>,
}

impl ChatServer {
    pub fn new(visitor_count: Arc<AtomicUsize>) -> ChatServer {
        // default room
        ChatServer {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
            visitor_count,
        }
    }
}

impl ChatServer {
    /// Send message to all users in the room
    fn send_message(&self, receiver: &str, message: &str) {
        if let Some(sessions) = self.rooms.get(receiver) {
            for id in sessions {
                if let Some(addr) = self.sessions.get(id) {
                    let _ = addr.do_send(Message(message.to_owned()));
                }
            }
        }
    }

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
    /// We are going to use simple Context, we just need ability to communicate with other actors.
    type Context = Context<Self>;
}

/// Handler for Connect message.
/// Register a new session and assign unique id to this session
impl Handler<Connect> for ChatServer {
    type Result = ();
    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        self.visitor_count.fetch_add(1, Ordering::SeqCst);
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Disconnect: {msg:?}");

        // remove address
        if self.sessions.remove(&msg.session_id).is_some() {
            // remove session from all rooms
            for (id, sessions) in &mut self.rooms {
                sessions.remove(id);
            }
        }
        self.clean_rooms();

        if self.visitor_count.load(Ordering::SeqCst) > 0 {
            self.visitor_count.fetch_sub(1, Ordering::SeqCst);
        }
    }
}

/// Gets called when connection is established, returns the session ID after
/// registering the session and joining its own room
impl Handler<Session> for ChatServer {
    type Result = String;

    fn handle(&mut self, msg: Session, _: &mut Context<Self>) -> Self::Result {
        let id = msg.session_id;
        self.sessions.insert(id.clone(), msg.address);
        self.rooms
            .entry(id.to_owned())
            .or_insert_with(HashSet::new)
            .insert(id.clone());
        println!("Sess: {:?}", self.sessions);
        println!("Rooms: {:?}", self.rooms);
        id
    }
}

impl Handler<Users> for ChatServer {
    type Result = Vec<String>;
    fn handle(&mut self, _: Users, _: &mut Context<Self>) -> Self::Result {
        let mut users = vec![];
        for session_id in self.sessions.clone().into_keys() {
            users.push(session_id);
        }
        users
    }
}

/// Handler for Message message.
impl<T> Handler<ClientMessage<T>> for ChatServer {
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
            //println!("MSG: {:?}", &msg);
            println!("ROOMS: {:?}", self.rooms);
            println!("SESSIONS: {:?}", self.sessions);
            if let Some(rooms) = self.rooms.get(&msg.session_id) {
                if let Some(receiver) = rooms.get(&chat_message.receiver_id) {
                    println!("SENDING MSG TO: {:?}", self.sessions.get(receiver));
                    self.send_message(receiver, &body);
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
            self.send_message(&room, "Someone disconnected");
        }

        self.rooms
            .entry(name.clone())
            .or_insert_with(HashSet::new)
            .insert(id.clone());

        self.send_message(&name, "Someone connected");
    }
}
