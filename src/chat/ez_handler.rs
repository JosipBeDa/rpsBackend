use super::models::{ClientMessage, Session, Users};
use super::session::WsChatSession;
use actix::prelude::*;
use actix_web_actors::ws::WebsocketContext;
use serde::de::DeserializeOwned;

/// The ultimate handler function for the ezSocket protocol
type ByteString = String;
pub fn handle<T>(
    text: ByteString,
    session: &mut WsChatSession,
    ctx: &mut WebsocketContext<WsChatSession>,
) {
    let message: ClientMessage<T> = serde_json::from_str(&text.trim()).unwrap();
    match message.header.as_ref() {
        "session" => {
            let address = ctx.address();
            session
                .addr
                .send(Session {
                    session_id: message.session_id.clone(),
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
                // Wait stops the actor from processing anything else until the future is done
                .wait(ctx)
        }
        "users" => {
            session
                .addr
                .send(Users)
                .into_actor(session)
                .then(|result, actor, ctx| {
                    match result {
                        Ok(users) => {
                            ctx.text(serde_json::to_string(&users).unwrap());
                        }
                        _ => println!("Something is wrong"),
                    }
                    fut::ready(())
                })
                .wait(ctx);

            ctx.text("joined");
        }
        _ => println!("hello"),
    }
}

fn generate_data_message<T>(
    session: &WsChatSession,
    data: Option<T>,
    header: &str,
) -> ClientMessage<T>
where
    T: DeserializeOwned,
{
    ClientMessage {
        session_id: session.id,
        header: header.to_string(),
        data,
        body: None,
    }
}
