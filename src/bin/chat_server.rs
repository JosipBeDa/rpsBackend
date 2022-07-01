use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

use actix::*;
use actix_files::{Files, NamedFile};
use actix_web::{middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use core::result::Result;
use lib::chat::*;
use lib::models::authentication::AuthenticationError;
use lib::models::custom_error::CustomError;
use lib::services::jwt;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct SessionID {
    id: String,
}

async fn index() -> impl Responder {
    NamedFile::open_async("./static/index.html").await.unwrap()
}

/// Entry point for our websocket route
async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    server: web::Data<Addr<server::ChatServer>>,
) -> Result<HttpResponse, CustomError> {
    if let Some(token) = req.cookie("Authorization") {
        match jwt::verify(token.value()) {
            Ok(user_id) => {
                match ws::WsResponseBuilder::new(
                    session::WsChatSession {
                        id: user_id.clone(),
                        hb: Instant::now(),
                        room: user_id,
                        name: None,
                        addr: server.get_ref().clone(),
                    },
                    &req,
                    stream,
                )
                .protocols(&["ezSocket"])
                .start()
                {
                    Ok(response) => Ok(response),
                    Err(e) => Err(CustomError::ActixError(e)),
                }
            }
            Err(e) => return Err(e),
        }
    } else {
        Err(AuthenticationError::Unauthorized.into())
    }
}

/// Displays state
async fn get_count(count: web::Data<AtomicUsize>) -> impl Responder {
    let current_count = count.load(Ordering::SeqCst);
    format!("Visitors: {current_count}")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // set up applications state
    // keep a count of the number of visitors
    let app_state = Arc::new(AtomicUsize::new(0));

    // start chat server actor
    let server = server::ChatServer::new(app_state.clone()).start();

    //log::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(app_state.clone()))
            .app_data(web::Data::new(server.clone()))
            .service(web::resource("/").to(index))
            .route("/count", web::get().to(get_count))
            .route("/ws", web::get().to(chat_route))
            .service(Files::new("/static", "./static"))
            .wrap(Logger::default())
    })
    .workers(2)
    .bind(("127.0.0.1", 5000))?
    .run()
    .await
}
