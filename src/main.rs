use axum::{
    Json, RequestPartsExt, Router,
    extract::{
        FromRequest, FromRequestParts, MatchedPath, Request, State, rejection::JsonRejection,
    },
    http::StatusCode,
    routing::{get, post},
};
use diesel::prelude::*;
use diesel_async::{
    AsyncPgConnection, RunQueryDsl,
    pooled_connection::{AsyncDieselConnectionManager, bb8},
};
use dotenvy::dotenv;
use redis::{AsyncTypedCommands, aio::MultiplexedConnection};
use std::{env, sync::Arc};
use tokio::signal;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::{
    config::Config,
    context::AuthContext,
    error::{Report, Result},
    models::TokenDetails,
};

pub mod auth;
pub mod config;
pub mod context;
pub mod crypto;
pub mod error;
pub mod middleware;
pub mod models;
pub mod schema;

pub type Pool = bb8::Pool<AsyncPgConnection>;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool,
    pub auth: AuthContext,
    pub redis: MultiplexedConnection,
}

impl AppState {
    pub async fn store_refresh_token(&self, token_details: &TokenDetails) -> Result<(), Report> {
        let mut conn = self.redis.clone();
        let key = format!("refresh_token:{}", token_details.token_id);
        let value = serde_json::to_string(token_details)?;

        if let Some(expires_in) = token_details.expires_in {
            let ttl = (expires_in - chrono::Utc::now().timestamp()) as u64;
            conn.set_ex(&key, &value, ttl).await?;
        } else {
            conn.set(&key, &value).await?;
        }

        Ok(())
    }

    pub async fn revoke_refresh_token(&self, token_id: Uuid) -> Result<(), Report> {
        let mut conn = self.redis.clone();
        let key = format!("refresh_token:{}", token_id);

        conn.del(&key).await?;

        Ok(())
    }

    pub async fn try_from(config: &Config) -> Result<Self, Report> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let pg_config =
            AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_url);
        let pool = bb8::Pool::builder().build(pg_config).await.unwrap();

        let redis = config.redis().multiplexed_connection().await?;

        let auth = AuthContext {
            access: config.auth().access().try_into()?,
            refresh: config.auth().refresh().try_into()?,
        };

        Ok(Self { pool, auth, redis })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    println!("{}", env!("CARGO_CRATE_NAME"));

    let config = Config::load()?;

    config.logger().setup()?;

    let state = AppState::try_from(&config).await?;

    let app = Router::new()
        .route("/api/user/list", get(auth::list_users))
        .route("/api/user/create", post(auth::create_user))
        .nest("/api/auth", auth::router(&state))
        .route("/", get(root))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(middleware::make_span_with)
                .on_request(middleware::on_request)
                .on_response(middleware::on_response)
                .on_failure(middleware::on_failure),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(Into::into)
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

pub fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
