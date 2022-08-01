use super::app::AppState;
use crate::models::error::GlobalError;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use crate::config::config::Config;
pub type PgPool = Pool<ConnectionManager<PgConnection>>;

// Create connection pool for global application use
pub fn establish_pool_connection() -> PgPool {
    let config = Config::from_env().expect("Couldn't build config");
    let database_url = config.get_db_url();
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    // min_idle VERY IMPORTANT for sanity, if not set the server will use the start with YOLO method
    Pool::builder()
        .min_idle(Some(1))
        .build(manager)
        .expect("Failed to create pool.")
}

pub fn connect(
    state: &AppState,
) -> Result<PooledConnection<ConnectionManager<PgConnection>>, GlobalError> {
    state.db_pool.get().map_err(|_| GlobalError::R2D2Error)
}
