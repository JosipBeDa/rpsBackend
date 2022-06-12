pub mod schema;
pub mod models;

#[macro_use]
extern crate diesel;
extern crate dotenv;

pub struct AuthForm {
    pub username: String,
    pub password: String
}

pub struct AuthResponse {
    ok: bool,

}

pub enum CustomError {
    DieselError(diesel::result::Error)
}