use rps_backend::models::user::{NewUser, User};
use rps_backend::{AuthForm, AuthResponse, CustomError};
use crate::state::app::AppState;
use actix_web::{web, web::Json, Responder};

pub async fn handler(user: web::Form<AuthForm>, state: web::Data<AppState>) -> Result<Json<AuthResponse>, CustomError> {
    let connection = match state.db_pool.get() {
        Ok(conn) => conn,
        Err(e) => return Err(CustomError::DieselError(e))
    };
    if User::find_by_uname(&connection, &user.username) != Ok(None) {

    }
}