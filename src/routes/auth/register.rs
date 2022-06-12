use crate::models::authentication::{AuthForm, AuthResponse};
use crate::models::custom_error::CustomError;
use crate::models::user::{NewUser, User};
use crate::services::jwt;
use crate::state::app::AppState;
use actix_web::{web, web::Json};

pub async fn handler(
    user: web::Form<AuthForm>,
    state: web::Data<AppState>,
) -> Result<Json<AuthResponse>, CustomError> {
    //Connect to the db pool
    let connection = match state.db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return Err(CustomError::R2D2Error),
    };
    //If the username is already taken
    if let Ok(Some(_)) = User::find_by_uname(&connection, &user.username) {
        return Err(CustomError::AlreadyExistsError);
    }
    let new_user = NewUser::create_and_store(&connection, &user.username, &user.password)?;

    let (token, exp_in) = jwt::generate_jwt(&new_user.id, &state.priv_key)?;

    Ok(Json(AuthResponse::new(true, new_user, token, exp_in)))
}
