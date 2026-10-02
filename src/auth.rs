use axum::{Json, extract::State, http::StatusCode};
use diesel::prelude::*;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

use crate::{crypto::generate_password_hash, internal_error, models::*, schema::users};

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
