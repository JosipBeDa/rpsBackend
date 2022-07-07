use crate::models::custom_error::CustomError;
use super::models::{ClientMessage, MessageData, Session, Users, LoL, LeL};
use super::session::WsChatSession;
use actix::prelude::*;
use actix_web_actors::ws::WebsocketContext;
use serde::{Serialize, de::DeserializeOwned};

/// The ultimate handler function for the ezSocket protocol
pub fn handle<T>(
    text: String,
    session: &mut WsChatSession,
    context: &mut WebsocketContext<WsChatSession>,
) where T: Serialize + DeserializeOwned {
    let message: ClientMessage<T> = serde_json::from_str(&text.trim()).expect("Bad message");
    match message.header.as_ref() {
        "session" => {
            let address = context.address();
            session
                .addr
                .send(Session {
                    session_id: session.id.clone(),
                    address: address.recipient(),
                })
                .into_actor(session)
                .then(|result, actor, context| {
                    // Actor is the established session and result is the result of the
                    // underlying chat server handler, in this case Session
                    match result {
                        Ok(id) => {
                            let message = generate_data_message::<String>(MessageData::String(id), "session").unwrap();
                            context.text(message);
                        }
                        _ => println!("Something is wrong"),
                    }
                    fut::ready(())
                })
                .wait(context)
            // Wait stops the actor from processing anything else until the future is done
        }
        "users" => {
            session
                .addr
                .send(Users)
                .into_actor(session)
                .then(|result, actor, context| {
                    match result {
                        Ok(users) => {
                            let msg =
                                generate_data_message(MessageData::List(users), "users")
                                    .unwrap();
                            context.text(msg);
                        }
                        _ => println!("Something is wrong"),
                    }
                    fut::ready(())
                })
                .wait(context);
        }
        "lol" => {
            session.addr.do_send(LoL{});
            session.addr.do_send(LeL{});
        }
        _ => println!("Bad message"),
    }
}

pub fn generate_data_message<T>(
    data: MessageData<T>,
    header: &str,
) -> Result<String, CustomError>  where T: Serialize {
    serde_json::to_string(&ClientMessage {
        header: header.to_string(),
        data: Some(data),
        body: None,
    })
    .map_err(|e| CustomError::SerdeError(e))
}
