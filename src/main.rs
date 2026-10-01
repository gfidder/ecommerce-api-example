use axum::{
    Json, RequestPartsExt, Router,
    extract::{FromRequestParts, State},
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use deadpool_diesel::{Manager, Pool};
use diesel::prelude::*;
use dotenvy::dotenv;
use rand::distr::{Alphanumeric, SampleString};
use serde::Deserialize;
use std::env;
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::schema::users;

use self::models::*;

pub mod crypto;
pub mod models;
pub mod schema;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer().without_time())
        .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager =
        deadpool_diesel::sqlite::Manager::new(database_url, deadpool_diesel::Runtime::Tokio1);
    let pool = deadpool_diesel::sqlite::Pool::builder(manager)
        .build()
        .unwrap();

    let app = Router::new()
        .route("/api/user/list", get(list_users))
        .route("/api/user/create", post(create_user))
        .route("/", get(root))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler")
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c  => {},
        _ = terminate => {}
    }
}

async fn root() -> &'static str {
    "Hello World!"
}

async fn create_user(
    State(pool): State<deadpool_diesel::sqlite::Pool>,
    Json(new_user): Json<NewUserQuery>,
) -> Result<Json<User>, (StatusCode, String)> {
    let salt = Alphanumeric.sample_string(&mut rand::rng(), 16);
    let pre_password_hash = salt.clone() + &new_user.password;

    let new_user_vals = NewUser {
        name: new_user.name,
        salt,
        password_hash: new_user.password,
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
struct NewUserQuery {
    name: String,
    password: String,
    email: String,
    first_name: String,
    last_name: String,
}

async fn list_users(
    State(pool): State<deadpool_diesel::sqlite::Pool>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    let res = conn
        .interact(|conn| User::query().load(conn))
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;

    Ok(Json(res))
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
