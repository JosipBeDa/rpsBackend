use super::models::{ChatMessage, ClientMessage, MessageData};
use super::session::WsChatSession;
use crate::models::custom_error::CustomError;
use actix_web_actors::ws::WebsocketContext;
use serde::{de::DeserializeOwned, Serialize};
use tracing::log::warn;

/// The ultimate handler function for the ezSocket protocol
pub fn handle<T>(
    text: String,
    session: &mut WsChatSession,
    context: &mut WebsocketContext<WsChatSession>,
) where
    T: Serialize + DeserializeOwned,
{
    let (header, data) = parse_message::<ChatMessage>(text);
    match header.as_ref() {
        "chat_message" => {
            if let MessageData::ChatMessage(chat_message) = data {
                let client_message = ClientMessage::<ChatMessage> {
                    header,
                    data: MessageData::ChatMessage(chat_message),
                };
                session.address.do_send(client_message);
            }
        }
        _ => warn!("Bad message"),
    }
}

pub fn generate_message<T>(data: MessageData<T>, header: &str) -> Result<String, CustomError>
where
    T: Serialize,
{
    serde_json::to_string(&ClientMessage {
        header: header.to_string(),
        data,
    })
    .map_err(|e| CustomError::SerdeError(e))
}

pub fn parse_message<T: DeserializeOwned + Serialize>(message: String) -> (String, MessageData<T>) {
    let message: ClientMessage<T> = serde_json::from_str(&message.trim()).unwrap();
    (message.header, message.data)
}
