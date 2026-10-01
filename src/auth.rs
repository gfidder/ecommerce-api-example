use axum::{Json, extract::State, http::StatusCode};
use diesel::prelude::*;
use rand::rngs::StdRng;
use serde::Deserialize;

use crate::{crypto::generate_password_hash, internal_error, models::*, schema::users};

#[axum::debug_handler]
pub async fn create_user(
    State(pool): State<deadpool_diesel::sqlite::Pool>,
    Json(new_user): Json<NewUserQuery>,
) -> Result<Json<User>, (StatusCode, String)> {
    let mut rand: StdRng = rand::make_rng();

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

    let conn = pool.get().await.map_err(internal_error)?;
    let res = conn
        .interact(|conn| {
            diesel::insert_into(users::table)
                .values(new_user_vals)
                .returning(User::as_returning())
                .get_result(conn)
        })
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;

    Ok(Json(res))
}

#[derive(Deserialize)]
pub struct NewUserQuery {
    name: String,
    password: String,
    email: String,
    first_name: String,
    last_name: String,
}
