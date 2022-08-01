use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::middleware::Logger;
use actix_web::{cookie::Key, web, web::Data, App, HttpServer};
use cookie::SameSite;
use lib::application;
use lib::config::config::Config;
use lib::state;
use tracing::info;
use env_logger::Env;

pub async fn hello_world() -> impl actix_web::Responder {
    "Sanity works!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //Used for logging
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    //Initialize the application state containing the reqwest client, DB pool and chat actor
    let state = Data::new(state::app::AppState::initialize());
    let config = Config::from_env().expect("Couldn't construct configuration");
    let session_secret = Key::generate();

    info!(
        "Starting server on {}:{}",
        config.get_address().0,
        config.get_address().1
    );

    HttpServer::new(move || {
        App::new()
        .app_data(state.clone())
        .configure(application::init::setup_routes)
        .route("/hello", web::get().to(hello_world))
        .wrap(
            SessionMiddleware::builder(CookieSessionStore::default(), session_secret.clone())
            .cookie_same_site(SameSite::None)
            .build(),
        )
        .wrap(application::init::setup_cors())
        .wrap(Logger::default())
    })
    .bind(config.get_address())?
    .run()
    .await
}
