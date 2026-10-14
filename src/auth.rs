use std::sync::LazyLock;

use axum::{
    Extension, Json, Router,
    body::Body,
    extract::State,
    http::{
        HeaderValue, StatusCode,
        header::{AUTHORIZATION, SET_COOKIE},
    },
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use axum_extra::extract::cookie;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    AppState,
    crypto::{self, generate_password_hash},
    error::Result,
    internal_error, middleware,
    models::*,
    schema::{session, users},
};

#[axum::debug_handler]
pub async fn create_user(
    State(state): State<AppState>,
    Json(new_user): Json<NewUserQuery>,
) -> Result<Json<NewUserResponse>, (StatusCode, String)> {
    let pool = state.pool;

    let mut rand: StdRng = rand::make_rng();
    let mut conn = pool.get().await.map_err(internal_error)?;

    let res: Vec<User> = User::query()
        .load(&mut conn)
        .await
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

    let result: User = diesel::insert_into(users::table)
        .values(new_user_vals)
        .returning(User::as_returning())
        .get_result(&mut conn)
        .await
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
    State(state): State<AppState>,
) -> Result<Json<Vec<ListUserResponse>>, (StatusCode, String)> {
    let pool = state.pool;

    let mut conn = pool.get().await.map_err(internal_error)?;
    let res = ListUserResponse::query()
        .load(&mut conn)
        .await
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

#[derive(Debug, Deserialize, Serialize, Clone)]
struct LoginUser {
    email: String,
    password: String,
}

async fn login(State(state): State<AppState>, Json(params): Json<LoginUser>) -> Result<Response> {
    let mut conn = state.pool.get().await?;

    tracing::debug!("Made it here");

    let user: User = User::query()
        .filter(users::email.eq(params.email))
        .first(&mut conn)
        .await
        .unwrap();

    crypto::verify_password(&params.password, &user.password_hash)?;

    let access_token = state.auth.access.generate_token(user.pid)?;
    let refresh_token = state.auth.refresh.generate_token(user.pid)?;

    state.store_refresh_token(&refresh_token).await?;

    let access_token = access_token.token.unwrap();
    let refresh_token = refresh_token.token.unwrap();

    let access_cookie = cookie::Cookie::build(("access_token", &access_token))
        .path("/")
        .http_only(true)
        .max_age(time::Duration::seconds(state.auth.access.exp))
        .same_site(cookie::SameSite::Lax);

    let refresh_cookie = cookie::Cookie::build(("refresh_token", &refresh_token))
        .path("/")
        .http_only(true)
        .max_age(time::Duration::seconds(state.auth.refresh.exp))
        .same_site(cookie::SameSite::Lax);

    let mut res = Response::builder().status(StatusCode::OK).body(Body::from(
        json!({
            "access_token": &access_token,
            "name": &user.name,
            // "created_at": user.
        })
        .to_string(),
    ))?;

    res.headers_mut()
        .append(AUTHORIZATION, HeaderValue::from_str(access_token.as_str())?);
    res.headers_mut().append(
        SET_COOKIE,
        HeaderValue::from_str(access_cookie.to_string().as_str())?,
    );
    res.headers_mut().append(
        SET_COOKIE,
        HeaderValue::from_str(refresh_cookie.to_string().as_str())?,
    );

    Ok(res)
}

async fn current(
    Extension(auth): Extension<TokenDetails>,
    State(state): State<AppState>,
) -> Result<Response> {
    let mut conn = state.pool.get().await?;

    let user: User = User::query()
        .filter(users::pid.eq(auth.user_pid))
        .first(&mut conn)
        .await
        .map_err(|_| crate::error::Error::Model(ModelError::EntityNotFound))?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "name": &user.name,
            "pid": &user.pid,
            "email": &user.email
        })),
    )
        .into_response())
}

#[derive(Serialize, HasQuery)]
#[diesel(table_name = session)]
pub struct LoginReturn {
    id: i32,
    expires_at: DateTime<Utc>,
    token: String,
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

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/login",
            post(login).route_layer(from_fn_with_state(
                state.clone(),
                middleware::auth_middleware,
            )),
        )
        .route(
            "/current",
            get(current)
                .route_layer(from_fn_with_state(
                    state.clone(),
                    middleware::auth_middleware,
                ))
                .route_layer(from_fn_with_state(
                    state.clone(),
                    middleware::refresh_middleware,
                )),
        )
        .with_state(state.clone())
}
