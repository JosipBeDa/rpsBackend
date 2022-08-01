use crate::routes;
use actix_cors::Cors;
use actix_web::{http, web};

/// Registers all routes contained in this function on the server
pub fn setup_routes(cfg: &mut web::ServiceConfig) {
    // POST /login
    cfg.service(web::resource("/login").route(web::post().to(routes::auth::login::handler)));
    // POST /register
    cfg.service(web::resource("/register").route(web::post().to(routes::auth::register::handler)));
    // GET /users
    cfg.service(
        web::resource("/users")
            .route(web::get().to(routes::users::handler))
            .wrap(crate::middleware::auth::LoggedGuard),
    );
    // GET /hof
    cfg.service(
        web::resource("/hof")
            .route(web::get().to(routes::hall_of_fame::handler))
            .wrap(crate::middleware::auth::LoggedGuard),
    );
    // GET /chat -- Upgrades to websocket on success, extracts user info from the authorization JWT so no need for LoggedGuard  
    cfg.service(
        web::resource("/chat")
            .route(web::get().to(routes::chat::handler))
    );
}

/// Return cors configuration
pub fn setup_cors() -> Cors {
    Cors::default()
        .allowed_origin("http://localhost:4200")
        .supports_credentials()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allowed_header(http::header::CONTENT_TYPE)
        .max_age(3600)
}
