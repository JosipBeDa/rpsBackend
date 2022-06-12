#[macro_use]
extern crate diesel;
extern crate bcrypt;
extern crate dotenv;
extern crate jsonwebtoken;

pub mod application;
pub mod state;
pub mod routes;
pub mod models;
pub mod schema;
pub mod crypto;
pub mod services;

use actix_web::middleware::Logger;
use actix_web::{web::Data, App, HttpServer};
use env_logger::Env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //Used for logging
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    //Initialize the application state containing the reqwest client and DB pool
    let app_state = Data::new(state::app::AppState::initialize());
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new(
                "%a %t '%r' %s %b '%{Referer}i' '%{User-Agent}i' %T",
            ))
            .app_data(app_state.clone())
            .configure(application::setup_routes)
            .wrap(application::setup_cors())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
