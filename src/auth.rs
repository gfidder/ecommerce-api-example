use std::sync::LazyLock;

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use diesel::prelude::*;
use jsonwebtoken::{DecodingKey, EncodingKey};
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    crypto::generate_password_hash,
    internal_error,
    models::*,
    schema::{session, users},
};

static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

#[axum::debug_handler]
pub async fn create_user(
    State(pool): State<deadpool_diesel::sqlite::Pool>,
    Json(new_user): Json<NewUserQuery>,
) -> Result<Json<NewUserResponse>, (StatusCode, String)> {
    let mut rand: StdRng = rand::make_rng();
    let conn = pool.get().await.map_err(internal_error)?;

    let res: Vec<User> = conn
        .interact(|conn| User::query().load(conn))
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;

    if res.iter().find(|&x| x.name == new_user.name).is_some() {
        return Err((StatusCode::BAD_REQUEST, "User already exists".into()));
    }

    if res.iter().find(|&x| x.email == new_user.email).is_some() {
        return Err((StatusCode::BAD_REQUEST, "Email is already in use".into()));
    }

    let (salt, hash) =
        generate_password_hash(&new_user.password, &mut rand).map_err(internal_error)?;

    let new_user_vals = NewUser {
        name: new_user.name,
        salt,
        password_hash: hash,
        email: new_user.email,
        first_name: new_user.first_name,
        last_name: new_user.last_name,
    };

    let result = conn
        .interact(|conn| {
            diesel::insert_into(users::table)
                .values(new_user_vals)
                .returning(User::as_returning())
                .get_result(conn)
        })
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;

    let response = NewUserResponse {
        id: result.id,
        name: result.name,
    };

    Ok(Json(response))
}

#[derive(Deserialize)]
pub struct NewUserQuery {
    name: String,
    password: String,
    email: String,
    first_name: String,
    last_name: String,
}

#[derive(Serialize)]
pub struct NewUserResponse {
    id: i32,
    name: String,
}

pub async fn list_users(
    State(pool): State<deadpool_diesel::sqlite::Pool>,
) -> Result<Json<Vec<ListUserResponse>>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    let res: Vec<ListUserResponse> = conn
        .interact(|conn| ListUserResponse::query().load(conn))
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;

    Ok(Json(res))
}

#[derive(Serialize, HasQuery)]
#[diesel(table_name = users)]
pub struct ListUserResponse {
    id: i32,
    name: String,
    email: String,
    first_name: String,
    last_name: String,
}

pub async fn login(State(pool): State<deadpool_diesel::sqlite::Pool>) {}

#[derive(Serialize, HasQuery)]
#[diesel(table_name = session)]
pub struct LoginReturn {
    id: i32,
    expires_at: String,
    token: String,
}

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong Credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing Credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token Creation Error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid Token"),
        };
        let body = Json(json!({
            "error": error_message
        }));
        (status, body).into_response()
    }
}

#[derive(Debug)]
pub enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
}
