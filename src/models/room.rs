use uuid::Uuid;
use diesel::{Queryable, Insertable};

#[derive(Queryable)]
pub struct Room {
    id: Uuid,
    name: String,
    password: Option<String>
}

use crate::schema::rooms;
#[derive(Insertable)]
#[table_name="rooms"]
pub struct NewRoom<'a> {
    id: &'a str,
    name: &'a str,
    password: Option<&'a str>
}