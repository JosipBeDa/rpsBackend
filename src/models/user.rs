use diesel::{prelude::*, ExpressionMethods, Insertable, Queryable, RunQueryDsl};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password: String,
}
pub use crate::schema::users;

#[derive(Insertable, Debug, Queryable)]
#[table_name = "users"]
pub struct NewUser {
    pub id: String,
    pub username: String,
    pub password: String,
}

impl User {
    pub fn find_by_uname(
        conn: &PgConnection,
        username: &str,
    ) -> Result<Option<User>, diesel::result::Error> {
        match users::table
            .filter(users::username.eq(username))
            .load::<User>(conn)
        {
            Ok(mut result) => Ok(result.pop()),
            Err(e) => Err(e),
        }
    }
}

use bcrypt::hash;
use diesel::PgConnection;

impl NewUser {
    pub fn create_and_store<'a>(
        conn: &PgConnection,
        username: &'a str,
        password: &'a str,
    ) -> Result<User, diesel::result::Error> {
        let hashed_pw = match hash(password, bcrypt::DEFAULT_COST) {
            Ok(hashed) => hashed,
            Err(e) => {
                println!("Hashing password error: {:?}", e);
                return Err(diesel::result::Error::__Nonexhaustive);
            }
        };

        let id = Uuid::new_v4().to_string();

        let new_user = Self {
            id: id,
            username: username.to_string(),
            password: hashed_pw,
        };

        diesel::insert_into(users::table)
            .values(&new_user)
            .get_result::<User>(conn)
    }
}
