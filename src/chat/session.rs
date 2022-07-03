use super::ez_handler;
use super::models::{
    ChatMessage, ClientMessage, Connect, Disconnect, Join, ListRooms, Message, Session, Users,
};
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
/// Clean empty rooms

#[derive(Debug)]
pub struct WsChatSession {
    /// Unique session id
    pub id: String,
    /// The username of the connected client
    pub username: String,
    /// Joined room
    pub room: String,
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    pub hb: Instant,
    /// Chat server
    pub addr: Addr<ChatServer>,
}

impl WsChatSession {
    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");
                // notify chat server
                act.addr.do_send(Disconnect {
                    session_id: act.id.clone(),
                });
                // stop actor
                ctx.stop();
                // don't try to send a ping
                return;
            }
            ctx.ping(b"");
        });
    }
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start.
    /// We register ws session with ChatServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // we'll start heartbeat process on session start.
        self.hb(ctx);
        print!("{}", "ACTOR ID CONNECT: ".yellow());
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
        // notify chat server
        print!("{}", "ACTOR ID DISCONNECT: ".yellow());
        println!("{:?}", self.id);
        self.addr.do_send(Disconnect {
            session_id: self.id.clone(),
        });
        Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<Message> for WsChatSession {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) {
        println!("HANDLER<MESSAGE> 4 CHATSESSION: {:?}", msg.0);
        ctx.text(msg.0);
    }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        // println!("WEBSOCKET MESSAGE: {msg:?}");
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                ez_handler::handle(text.to_string(), self, ctx);
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}
