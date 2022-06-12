use actix_cors::Cors;
use actix_web::{http, web};

/// Returns the routing config
pub fn setup_routes(cfg: &mut web::ServiceConfig) {
    // GET /
    //cfg.service(web::resource("/").route(web::get().to(routes::hello_world)));
}

/// Return cors configuration for the project
pub fn setup_cors() -> Cors {
    Cors::default()
        .send_wildcard()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
        .allowed_header(http::header::CONTENT_TYPE)
        .max_age(3600)
}
