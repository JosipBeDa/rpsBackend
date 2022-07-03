use crate::models::custom_error::CustomError;

use super::models::{ClientMessage, MessageData, Session, Users};
use super::session::WsChatSession;
use actix::prelude::*;
use actix_web_actors::ws::WebsocketContext;
use colored::Colorize;

/// The ultimate handler function for the ezSocket protocol
pub fn handle(
    text: String,
    session: &mut WsChatSession,
    ctx: &mut WebsocketContext<WsChatSession>,
) {
    let message: ClientMessage = serde_json::from_str(&text.trim()).expect("Bad message");
    match message.header.as_ref() {
        "session" => {
            let session_id = message.session_id;
            let address = ctx.address();
            session
                .addr
                .send(Session {
                    session_id,
                    address: address.recipient(),
                })
                .into_actor(session)
                .then(|result, actor, ctx| {
                    // Actor is the established session and result is the result of the
                    // underlying chat server handler, in this case Session
                    match result {
                        Ok(id) => {
                            actor.id = id;
                            ctx.text("Welcome");
                        }
                        _ => println!("Something is wrong"),
                    }
                    fut::ready(())
                })
                .wait(ctx)

            // Wait stops the actor from processing anything else until the future is done
        }
        "users" => {
            session
                .addr
                .send(Users)
                .into_actor(session)
                .then(|result, actor, ctx| {
                    match result {
                        Ok(users) => {
                            let msg =
                                generate_data_message(actor, MessageData::List(users), "users")
                                    .unwrap();
                            ctx.text(msg);
                        }
                        _ => println!("Something is wrong"),
                    }
                    fut::ready(())
                })
                .wait(ctx);
        }
        _ => println!("Bad message"),
    }
}

fn generate_data_message(
    session: &WsChatSession,
    data: MessageData,
    header: &str,
) -> Result<String, CustomError> {
    serde_json::to_string(&ClientMessage {
        session_id: session.id.clone(),
        header: header.to_string(),
        data: Some(data),
        body: None,
    })
    .map_err(|e| CustomError::SerdeError(e))
}
