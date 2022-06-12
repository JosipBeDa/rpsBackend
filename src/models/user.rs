use uuid::Uuid;
use diesel::{prelude::*, Queryable, Insertable, RunQueryDsl, ExpressionMethods};
use serde::Serialize;

#[derive(Queryable, PartialEq, Debug, Clone, Serialize)]
pub struct User {
    id: String,
    username: String,
    password: String
}

use crate::schema::users;

#[derive(Insertable, Debug, Queryable)]
#[table_name="users"]
pub struct NewUser {
    id: String,
    username: String,
    password: String
}

impl User {
    pub fn find_by_uname(conn: &PgConnection, username: &str) -> Result<Option<User>, diesel::result::Error> {
        match users::table.filter(users::username.eq(username)).load::<User>(conn) {
            Ok(mut result) => Ok(result.pop()),
            Err(e) => Err(e)
        }
    }
}

use diesel::PgConnection;
use bcrypt::hash;

impl NewUser {
    pub fn create_and_store(conn: &PgConnection, username: String, password: String) -> Result<User, diesel::result::Error> {
        let hashed_pw = match hash(password, bcrypt::DEFAULT_COST) {
            Ok(hashed) => hashed,
            Err(e) => {
                println!("Hashing password error: {:?}", e);
                return Err(diesel::result::Error::__Nonexhaustive)
            }
        };

        let id = Uuid::new_v4().to_string();

        let new_user = Self {
            id: id,
            username,
            password: hashed_pw
        };

        diesel::insert_into(users::table).values(&new_user).get_result::<User>(conn)
    }
}