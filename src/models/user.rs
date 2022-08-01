use super::error::GlobalError;
use crate::actors::chat::models::chat_user::ChatUser;
use crate::schema::users;
use bcrypt::hash;
use diesel::{
    ExpressionMethods, OptionalExtension, PgConnection, QueryDsl, Queryable, RunQueryDsl,
};
use serde::{Deserialize, Serialize};

#[derive(Queryable, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
}

#[derive(Insertable, Debug)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password: String,
}

impl User {
    pub fn find_by_uname(conn: &PgConnection, username: &str) -> Result<Option<User>, GlobalError> {
        users::table
            .filter(users::username.eq(username))
            .first::<User>(conn)
            .optional()
            .map_err(|e| GlobalError::DieselError(e))
    }
    pub fn find_by_id(conn: &PgConnection, id: &str) -> Result<Option<User>, GlobalError> {
        users::table
            .filter(users::id.eq(id))
            .first::<User>(conn)
            .optional()
            .map_err(|e| GlobalError::DieselError(e))
    }
    pub fn find_all(conn: &PgConnection, limit: i64) -> Result<Vec<User>, GlobalError> {
        users::table
            .limit(limit)
            .load(conn)
            .map_err(|e| GlobalError::DieselError(e))
    }
    /// Converts a User struct from the database to the user struct used by the chat server
    pub fn convert(self) -> ChatUser {
        ChatUser {
            id: self.id,
            username: self.username,
            connected: false,
        }
    }
}

impl<'a> NewUser<'a> {
    pub fn create_and_store(
        conn: &PgConnection,
        username: &'a str,
        password: &'a str,
    ) -> Result<User, GlobalError> {
        let hashed_pw = hash(password, bcrypt::DEFAULT_COST)?;

        let new_user = Self {
            username,
            password: hashed_pw,
        };

        diesel::insert_into(users::table)
            .values(&new_user)
            .get_result::<User>(conn)
            .map_err(|e| GlobalError::DieselError(e))
    }
}
