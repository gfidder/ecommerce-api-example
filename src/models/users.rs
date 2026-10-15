use crate::schema::users;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(HasQuery, Serialize)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub pid: Uuid,
    pub name: String,
    pub salt: String,
    pub password_hash: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
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
