use crate::models::authentication::{AuthForm, AuthResponse, AuthenticationError};
use crate::models::custom_error::CustomError;
use crate::models::user::{NewUser, User, ChatUser};
use crate::services::jwt;
use crate::state::app::AppState;
use actix_session::Session;
use actix_web::{web, Responder, HttpResponseBuilder};

pub async fn handler(
    user: web::Form<AuthForm>,
    state: web::Data<AppState>,
    session: Session,
) -> impl Responder {
    //Connect to the db pool
    let db_connection = match state.db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return Err(CustomError::R2D2Error),
    };
    match User::find_by_uname(&db_connection, &user.username) {
        Ok(None) => {
            let new_user = NewUser::create_and_store(&db_connection, &user.username, &user.password)?;
            let user = ChatUser {id: new_user.id, username: new_user.username, connected: true};
            let (token, _exp_in) = jwt::generate_jwt(user.to_string())?;
            Ok(HttpResponseBuilder::new(reqwest::StatusCode::OK)
                .insert_header(("Authorization", token))
                .json(AuthResponse::succeed(user, "Success!")))
        }
        Ok(Some(_)) => {
            Err(AuthenticationError::UserAlreadyExists.into())
        }
        Err(e) => Err(CustomError::DieselError(e)),
    }
}
