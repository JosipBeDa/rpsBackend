use super::ez_handler;
use super::models::{Connect, Disconnect, Message};
use super::server::ChatServer;
use crate::models::user::ChatUser;
use actix::prelude::*;
use actix_web_actors::ws;
use colored::Colorize;
use std::time::{Duration, Instant};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Session instance. Gets created each time a client connects.
/// 
/// Every session instance can conceptually be thought of as the intermediary
/// between the client and the server. The session actor receives messages
/// from the client and relays them to the chat server actor for processing in
/// the form of a future. The session then awaits the result of that future
/// and sends back a message to the client.
#[derive(Debug)]
pub struct WsChatSession {
    /// Unique session id obtained from the Authorization cookie
    pub id: String,
    /// The username of the connected client
    pub username: String,
    /// The currently joined room
    pub room: String,
    /// The heartbeat. A ping message gets sent every `HEARTBEAT_INTERVAL` seconds,
    /// if a pong isn't received for `CLIENT_TIMEOUT` seconds, drop the connection
    pub hb: Instant,
    /// The address of the chat server. Every session sends their messages to here for processing.
    pub addr: Addr<ChatServer>,
}

impl WsChatSession {
    /// Sends ping to the client every `HEARTBEAT_INTERVAL` seconds
    fn hb(&self, context: &mut ws::WebsocketContext<Self>) {
        context.run_interval(HEARTBEAT_INTERVAL, |act, context| {
            // Check if the duration is greater than the timeout
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // Heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");
                // Notify chat server
                act.addr.do_send(Disconnect {
                    session_id: act.id.clone(),
                });
                // Stop actor
                context.stop();
                // Don't try to send a ping
                return;
            }
            context.ping(b"");
        });
    }
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start.
    /// We register the session with ChatServer
    fn started(&mut self, context: &mut Self::Context) {
        // Start the heartbeat process on session start.
        self.hb(context);
        print!("{}", "ACTOR ID CONNECT: ".green());
        println!("{:?}", self.id);
        self.addr.do_send(Connect {
            user: ChatUser {
                id: self.id.clone(),
                username: self.username.clone(),
                connected: true,
            },
        })
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        print!("{}", "ACTOR ID DISCONNECT: ".red());
        println!("{:?}", self.id);
        self.addr.do_send(Disconnect {
            session_id: self.id.clone(),
        });
        Running::Stop
    }
}

/// The chat server can process messages 
impl Handler<Message> for WsChatSession {
    type Result = ();
    fn handle(&mut self, msg: Message, context: &mut Self::Context) {
        context.text(msg.0);
    }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, context: &mut Self::Context) {
        let msg = match msg {
            Ok(msg) => msg,
            Err(_) => {
                context.stop();
                return;
            }
        };

        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                context.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                ez_handler::handle::<String>(text.to_string(), self, context);
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(reason) => {
                context.close(reason);
                context.stop();
            }
            ws::Message::Continuation(_) => {
                context.stop();
            }
            ws::Message::Nop => (),
        }
    }
}
