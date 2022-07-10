use crate::models::custom_error::CustomError;
use crate::models::user::User;
use crate::state::app::AppState;
use actix_web::{web, web::Json};

pub async fn handler(
    state: web::Data<AppState>,
) -> Result<Json<Vec<User>>, CustomError> {
    let connection = match state.db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return Err(CustomError::R2D2Error),
    };
    match User::find_all(&connection, 10) {
        Ok(users) => Ok(Json(users)),
        Err(e) => Err(CustomError::DieselError(e)),
    }
}
