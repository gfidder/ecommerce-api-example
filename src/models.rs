use crate::schema::users;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(HasQuery, Serialize)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub id: i32,
    pub name: String,
    pub salt: String,
    pub password_hash: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
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

/// The token string deserializes to this struct
/// The `sub` field will be the user's pid
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TokenClaims {
    pub sub: String,
    pub id: String,
    pub exp: i64,
    pub iat: i64,
    pub nbf: i64,
}

/// This struct will let us store our token in Redis
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TokenDetails {
    pub token: Option<String>,
    pub token_id: Uuid,
    pub user_pid: Uuid,
    pub expires_in: Option<i64>,
}
