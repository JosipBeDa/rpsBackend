//! Every text message the `WsChatSession` stream handler receives is sent to this
//! handler for processing.
use super::models::{ChatMessage, ClientMessage, Join, MessageData, Read};
use super::session::WsChatSession;
use crate::chat::rps::{RPSMessage, RPS};
use crate::models::error::GlobalError;
use actix::prelude::*;
use actix_web_actors::ws::WebsocketContext;
use colored::Colorize;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use tracing::log::{info, warn};

/// Parses the text message to a `ClientMessage` struct and send the appropriate message to
/// the server.
pub fn handle<T>(
    text: String,
    session: &mut WsChatSession,
    context: &mut WebsocketContext<WsChatSession>,
) where
    T: Serialize + DeserializeOwned,
{
    let header = get_header(text.clone());
    info!("{}{:?}", "GOT HEADER : ".yellow(), header);
    match header.as_ref() {
        "chat_message" => {
            let message = parse_message::<ChatMessage>(text);
            if let MessageData::ChatMessage(chat_message) = message.data.clone() {
                let client_message = ClientMessage::<ChatMessage> {
                    header: message.header.clone(),
                    data: MessageData::ChatMessage(chat_message),
                };
                session.address.do_send(client_message);
            }
        }
        "join" => {
            let message = parse_message::<Join>(text);
            if let MessageData::Join(message) = message.data {
                session
                    .address
                    .send(Join {
                        id: message.id,
                        room_id: message.room_id,
                    })
                    .into_actor(session)
                    .then(|res, _, ctx| {
                        match res {
                            Ok(messages) => {
                                if messages.len() > 0 {
                                    ctx.text(
                                        generate_message("read", MessageData::List(messages))
                                            .unwrap(),
                                    );
                                }
                            }
                            Err(e) => warn!("SOMETHING WENT WRONG : {:?}", e),
                        }
                        fut::ready(())
                    })
                    .wait(context)
            }
        }
        "read" => {
            let message = parse_message::<ChatMessage>(text);
            if let MessageData::List::<ChatMessage>(messages) = message.data {
                session.address.do_send(Read { messages })
            }
        }
        "lol" => context.text(
            generate_message::<String>("lel", MessageData::String(String::from("lel"))).unwrap(),
        ),
        "rps" => {
            let message = parse_message::<RPSMessage>(text);
            info!("{}{:?}", "GOT RPS MESSAGE : ".purple(), message);
            if let MessageData::RPS(msg) = message.data {
                session
                    .rps_address
                    .send(msg)
                    .into_actor(session)
                    .then(|res, _, ctx| {
                        match res {
                            Ok(rps_game) => ctx.text(
                                generate_message::<RPS>("rps", MessageData::RPSState(rps_game))
                                    .unwrap(),
                            ),
                            Err(e) => warn!("SOMETHING WENT WRONG : {:?}", e),
                        }
                        fut::ready(())
                    })
                    .wait(context)
            }
        }
        _ => warn!("Bad message"),
    }
}

/// Generate a `ClientMessage` with the given data.
pub fn generate_message<T>(header: &str, data: MessageData<T>) -> Result<String, GlobalError>
where
    T: Serialize,
{
    serde_json::to_string(&ClientMessage {
        header: header.to_string(),
        data,
    })
    .map_err(|e| GlobalError::SerdeError(e))
}

/// Parses text to `ClientMessage`
pub fn parse_message<T: DeserializeOwned + Serialize>(message: String) -> ClientMessage<T> {
    info!("GOT MESSAGE PARSING : {}", message);
    serde_json::from_str::<ClientMessage<T>>(&message.trim()).unwrap()
}

pub fn get_header<'a>(s: String) -> String {
    let message: Value = serde_json::from_str(&s).unwrap();
    let header = &message["header"];
    header.as_str().expect("Couldn't parse header").to_string()
}
