#[macro_use]
extern crate diesel;
extern crate bcrypt;
extern crate dotenv;
extern crate jsonwebtoken;
extern crate tracing;

pub mod application;
pub mod config;
pub mod crypto;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod schema;
pub mod services;
pub mod state;
pub mod actors;

pub const TOKEN_DURATION: cookie::time::Duration = cookie::time::Duration::hours(3);
