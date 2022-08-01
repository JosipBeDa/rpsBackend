use super::error::GlobalError;
use crate::schema::hall_of_fame;
use diesel::{ExpressionMethods, OptionalExtension, PgConnection, QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Queryable, Debug, Clone, PartialEq, Serialize, Deserialize, AsChangeset)]
#[table_name = "hall_of_fame"]
pub struct HallOfFameEntry {
    id: i32,
    user_id: String,
    score: i32,
}

impl HallOfFameEntry {
    pub fn find_one<'a>(
        conn: &PgConnection,
        player_id: &'a str,
    ) -> Result<Option<HallOfFameEntry>, GlobalError> {
        hall_of_fame::table
            .filter(hall_of_fame::user_id.eq(player_id))
            .first::<HallOfFameEntry>(conn)
            .optional()
            .map_err(|e| GlobalError::DieselError(e))
    }

    pub fn find_all(conn: &PgConnection) -> Result<Vec<HallOfFameEntry>, GlobalError> {
        hall_of_fame::table
            .order(hall_of_fame::score.desc())
            .load(conn)
            .map_err(|e| GlobalError::DieselError(e))
    }

    pub fn incr_score(&mut self) {
        self.score += 1;
    }
}

#[derive(Insertable, Debug)]
#[table_name = "hall_of_fame"]
pub struct NewHoFEntry<'a> {
    user_id: &'a str,
    score: i32,
}

impl<'a> NewHoFEntry<'a> {
    pub fn upsert(conn: &PgConnection, user_id: &'a str) -> Result<usize, GlobalError> {
        if let Some(mut hof_entry) = HallOfFameEntry::find_one(conn, user_id)? {
            info!("Found HoF Entry : {:?}", hof_entry);
            hof_entry.incr_score();
            diesel::update(hall_of_fame::table)
                .filter(hall_of_fame::user_id.eq(user_id))
                .set(&hof_entry)
                .execute(conn)
                .map_err(|e| GlobalError::DieselError(e))
        } else {
            info!("No HoF entry found, creating with : {}", user_id);
            diesel::insert_into(hall_of_fame::table)
                .values(Self { user_id, score: 1 })
                .execute(conn)
                .map_err(|e| GlobalError::DieselError(e))
        }
    }
}
