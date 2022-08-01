use crate::models::error::GlobalError;
use crate::models::hall_of_fame::HallOfFameEntry;
use crate::state::app::AppState;
use actix_web::{web, web::Json};

pub async fn handler(state: web::Data<AppState>) -> Result<Json<Vec<HallOfFameEntry>>, GlobalError> {
    let connection = match state.db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return Err(GlobalError::R2D2Error),
    };
    match HallOfFameEntry::find_all(&connection) {
        Ok(entries) => Ok(Json(entries)),
        Err(e) => Err(e),
    }
}