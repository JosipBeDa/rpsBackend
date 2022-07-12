use crate::models::authentication::{AuthForm, AuthResponse};
use crate::models::error::{AuthenticationError, GlobalError};
use crate::models::user::{NewUser, User};
use crate::state::{db_pool, app::AppState};
use actix_web::{web, Responder};
use tracing::info;
use colored::Colorize;

pub async fn handler(user: web::Form<AuthForm>, state: web::Data<AppState>) -> impl Responder {
    info!("{}{:?}", "Registering user : ".cyan(), user);
    let db_connection = db_pool::connect(&state)?;
    let existing_user = User::find_by_uname(&db_connection, &user.username)?;
    if existing_user.is_some() {
        return Err(GlobalError::AuthenticationError(
            AuthenticationError::UserAlreadyExists,
        ));
    } else {
        let user = NewUser::create_and_store(&db_connection, &user.username, &user.password)?;
        Ok(AuthResponse::succeed_with_token(user))
    }
}
