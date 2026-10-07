mod config;
mod errors;
mod handlers;
mod models;
mod rate_limit;

use crate::config::AppConfig;
use crate::handlers::{health, redirect_to_url, shorten_url};
use crate::rate_limit::{RouteLimit, build_limiters, rate_limit_middleware};
use axum::routing::{get, post};
use axum::{Router, middleware};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: AppConfig,
}

async fn init_db(config: &AppConfig) -> anyhow::Result<SqlitePool> {
    let pool = SqlitePool::connect(&config.db_url).await?;
    println!("Connected to Sqlite");
    Ok(pool)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::from_env()?;
    let (shorten_limiter, redirect_limiter, health_limiter) =
        build_limiters(&config.rate_limit_config);

    let db = init_db(&config).await?;

    let state = AppState {
        db,
        config: config.clone(),
    };

    let app = Router::new()
        .route(
            "/health",
            get(health).layer(middleware::from_fn_with_state(
                RouteLimit {
                    limiter: health_limiter,
                    limit: config.rate_limit_config.health_limit,
                    window_seconds: config.rate_limit_config.health_window_seconds,
                },
                rate_limit_middleware,
            )),
        )
        .route(
            "/api/shorten",
            post(shorten_url).layer(middleware::from_fn_with_state(
                RouteLimit {
                    limiter: shorten_limiter,
                    limit: config.rate_limit_config.shorten_limit,
                    window_seconds: config.rate_limit_config.shorten_window_seconds,
                },
                rate_limit_middleware,
            )),
        )
        .route(
            "/{code}",
            get(redirect_to_url).layer(middleware::from_fn_with_state(
                RouteLimit {
                    limiter: redirect_limiter,
                    limit: config.rate_limit_config.redirect_limit,
                    window_seconds: config.rate_limit_config.redirect_window_seconds,
                },
                rate_limit_middleware,
            )),
        )
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    println!("Server running on http://{}", addr);

    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;

    Ok(())
}
