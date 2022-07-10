use crate::services::jwt;
use crate::state::app::AppState;
use crate::{chat::*, models::authentication::AuthResponse};
use actix_web::{web, HttpRequest, HttpResponseBuilder as Response, Responder};
use actix_web_actors::ws;
use reqwest::StatusCode;
use std::time::Instant;
use core::pin::Pin;

/// Entry point for our websocket route
pub async fn handler(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> impl Responder {
    if let Some(token) = req.cookie("Authorization") {
        match jwt::verify(token.value()) {
            Ok(chat_user) => {
                match ws::WsResponseBuilder::new(
                    session::WsChatSession {
                        id: chat_user.id.clone(),
                        username: chat_user.username,
                        room: chat_user.id,
                        heartbeat: Instant::now(),
                        address: Pin::new(&state.chat_server).get_ref().clone(),
                    },
                    &req,
                    stream,
                )
                .protocols(&["ezSocket"])
                .start()
                {
                    Ok(response) => response,
                    Err(_) => Response::new(StatusCode::INTERNAL_SERVER_ERROR)
                        .json(AuthResponse::fail("Internal server Error")),
                }
            }
            Err(_) => Response::new(StatusCode::INTERNAL_SERVER_ERROR)
                .json(AuthResponse::fail("Internal server Error")),
        }
    } else {
        Response::new(StatusCode::UNAUTHORIZED).json(AuthResponse::fail("Unauthorized"))
    }
}
