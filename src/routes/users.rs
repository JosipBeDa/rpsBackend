use crate::models::error::GlobalError;
use crate::models::user::User;
use crate::state::app::AppState;
use actix_web::{web, web::Json};

pub async fn handler(
    state: web::Data<AppState>,
) -> Result<Json<Vec<User>>, GlobalError> {
    let connection = match state.db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return Err(GlobalError::R2D2Error),
    };
    match User::find_all(&connection, 10) {
        Ok(users) => Ok(Json(users)),
        Err(e) => Err(GlobalError::DieselError(e)),
    }
}
