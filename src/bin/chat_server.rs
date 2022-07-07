use actix::*;
use actix_files::{Files, NamedFile};
use actix_web::{
    middleware::Logger, web, App, HttpRequest, HttpResponseBuilder as Response, HttpServer,
    Responder,
};
use actix_web_actors::ws;
use lib::services::jwt;
use lib::{chat::*, models::authentication::AuthResponse};
use reqwest::StatusCode;
use std::time::Instant;

async fn index() -> impl Responder {
    NamedFile::open_async("./static/index.html").await.unwrap()
}

/// Entry point for our websocket route
async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    server: web::Data<Addr<server::ChatServer>>,
) -> impl Responder {
    if let Some(token) = req.cookie("Authorization") {
        match jwt::verify(token.value()) {
            Ok(chat_user) => {
                match ws::WsResponseBuilder::new(
                    session::WsChatSession {
                        id: chat_user.id.clone(),
                        username: chat_user.username,
                        room: chat_user.id,
                        hb: Instant::now(),
                        addr: server.get_ref().clone(),
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // start chat server actor
    let server = server::ChatServer::new().start();
    //log::info!("starting HTTP server at http://localhost:8080");
    lib::chat::server::reg();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server.clone()))
            .service(web::resource("/").to(index))
            .route("/ws", web::get().to(chat_route))
            .service(Files::new("/static", "./static"))
            .wrap(Logger::default())
    })
    .workers(2)
    .bind(("127.0.0.1", 5000))?
    .run()
    .await
}
