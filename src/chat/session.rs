//! The session actor.
use super::ez_handler;
use super::models::messages::{Connect, Disconnect, Message};
use crate::rps::manager::RPSManager;
use super::server::ChatServer;
use crate::models::user::ChatUser;
use actix::prelude::*;
use actix_web_actors::ws;
use colored::Colorize;
use std::time::{Duration, Instant};
use tracing::info;
use tracing::log::warn;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Session instance. Gets created each time a client connects and communicates
/// with `ChatServer` through the `ez_handler`.
///
/// Depending on the type of send, the session actor awaits the result of
/// the message with `send` or just blindly send it with `do_send` without awaiting.
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
    pub heartbeat: Instant,
    /// The address of the chat server. Every session sends their messages to here for processing.
    pub address: Addr<ChatServer>,
    /// The address of the RPS Manager
    pub rps_address: Addr<RPSManager>,
}

impl WsChatSession {
    /// Sends a ping to the client every `HEARTBEAT_INTERVAL` seconds
    fn hb(&self, context: &mut ws::WebsocketContext<Self>) {
        context.run_interval(HEARTBEAT_INTERVAL, |actor, context| {
            // Check if the duration is greater than the timeout
            if Instant::now().duration_since(actor.heartbeat) > CLIENT_TIMEOUT {
                // Heartbeat timed out
                warn!("Websocket Client heartbeat failed, disconnecting!");
                // Notify chat server
                actor.address.do_send(Disconnect {
                    session_id: actor.id.clone(),
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

    /// Called on actor start, sends a `Connect` message to the server.
    fn started(&mut self, context: &mut Self::Context) {
        // Start the heartbeat process on session start.
        self.hb(context);
        info!("{}{:?}", "ACTOR STARTED -- ID : ".green(), self.id);

        let address = context.address().recipient();
        let message = Connect {
            user: ChatUser {
                id: self.id.clone(),
                username: self.username.clone(),
                connected: true,
            },
            address,
        };
        self.address.do_send(message.clone());
        self.rps_address.do_send(message);
    }

    /// Called on actor stop, sends a `Disconnect` message to the server.
    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        info!("{}{:?}", "ACTOR STOPPING -- ID : ".red(), self.id);
        self.address.do_send(Disconnect {
            session_id: self.id.clone(),
        });
        Running::Stop
    }
}

/// The session actor implements a handler only for the message type, which is
/// ultimately always going to be JSON. It simply sends a text frame
/// of that JSON to the client.
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
                self.heartbeat = Instant::now();
                context.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.heartbeat = Instant::now();
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
