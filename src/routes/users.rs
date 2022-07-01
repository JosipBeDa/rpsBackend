use crate::models::custom_error::CustomError;
use crate::models::user::User;
use crate::state::app::AppState;
use actix_session::Session;
use actix_web::{web, web::Json};

pub async fn handler(
    state: web::Data<AppState>,
    session: Session,
) -> Result<Json<Vec<User>>, CustomError> {
    if let Some(count) = session.get::<i32>("counter").unwrap() {
        // modify the session state
        session.insert("counter", count + 1).unwrap();
    } else {
        session.insert("counter", 1 as i32).unwrap();
    }
    let connection = match state.db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return Err(CustomError::R2D2Error),
    };
    println!("{:?}", session.entries());
    match User::find_all(&connection, 10) {
        Ok(users) => Ok(Json(users)),
        Err(e) => Err(CustomError::DieselError(e)),
    }
}
