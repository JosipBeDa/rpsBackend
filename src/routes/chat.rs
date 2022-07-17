use crate::chat::*;
use crate::models::error::{AuthenticationError, GlobalError};
use crate::services::jwt;
use crate::state::app::AppState;
use actix_web::{web, HttpRequest, Responder};
use actix_web_actors::ws;
use core::pin::Pin;
use std::time::Instant;

/// Entry point for our websocket route
pub async fn handler(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> impl Responder {
    if let Some(token) = req.cookie("Authorization") {
        let chat_user = jwt::verify(token.value())?;
        ws::WsResponseBuilder::new(
            session::WsChatSession {
                id: chat_user.id.clone(),
                username: chat_user.username,
                room: chat_user.id,
                heartbeat: Instant::now(),
                address: Pin::new(&state.chat_server).get_ref().clone(),
                rps_address: Pin::new(&state.rps_manager).get_ref().clone(),
            },
            &req,
            stream,
        )
        .protocols(&["ezSocket"])
        .start()
        .map_err(|e| GlobalError::ActixError(e))
    } else {
        // No token
        Err(AuthenticationError::InvalidToken.into())
    }
}
