use axum::{
    Json, RequestPartsExt, Router,
    extract::{
        FromRequest, FromRequestParts, MatchedPath, Request, State, rejection::JsonRejection,
    },
    http::{StatusCode, request::Parts},
    middleware::{Next, from_fn},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use deadpool_diesel::{Manager, Pool};
use diesel::SqliteConnection;
use dotenvy::dotenv;
use std::{env, sync::Arc};
use tokio::signal;
use tower_http::trace::TraceLayer;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    config::Config,
    context::{AppContext, AuthContext},
    error::{Report, Result},
};

pub mod auth;
pub mod config;
pub mod context;
pub mod crypto;
pub mod error;
pub mod middleware;
pub mod models;
pub mod schema;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Manager<SqliteConnection>>,
    pub auth: AuthContext,
}

impl AppState {
    pub async fn try_from(config: &Config) -> Result<Self, Report> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let manager =
            deadpool_diesel::sqlite::Manager::new(database_url, deadpool_diesel::Runtime::Tokio1);
        let pool = deadpool_diesel::sqlite::Pool::builder(manager)
            .build()
            .unwrap();

        let auth = AuthContext {
            access: config.auth().access().try_into()?,
            refresh: config.auth().refresh().try_into()?,
        };

        Ok(Self { pool, auth })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // tracing_subscriber::registry()
    //     .with(
    //         tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
    //             format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
    //         }),
    //     )
    //     .with(tracing_subscriber::fmt::layer().without_time())
    //     .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager =
        deadpool_diesel::sqlite::Manager::new(database_url, deadpool_diesel::Runtime::Tokio1);
    let pool = deadpool_diesel::sqlite::Pool::builder(manager)
        .build()
        .unwrap();

    let config = Config::load()?;

    config.logger().setup()?;

    let _ctx = Arc::new(AppContext::try_from(&config).await?);

    let state = AppState::try_from(&config).await?;

    let app = Router::new()
        .route("/api/user/list", get(auth::list_users))
        .route("/api/user/create", post(auth::create_user))
        .route("/api/user/login", get(auth::login))
        .route("/", get(root))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|req: &Request| {
                    let method = req.method();
                    let uri = req.uri();

                    let matched_path = req
                        .extensions()
                        .get::<MatchedPath>()
                        .map(|matched_path| matched_path.as_str());

                    tracing::debug_span!("request", %method, %uri, matched_path)
                })
                .on_failure(()),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
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
