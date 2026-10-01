use crate::schema::users;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(HasQuery, Serialize)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    id: i32,
    name: String,
    salt: String,
    password_hash: String,
    email: String,
    first_name: String,
    last_name: String,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub name: String,
    pub salt: String,
    pub password_hash: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}
