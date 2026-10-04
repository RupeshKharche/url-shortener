mod config;
mod errors;
mod handlers;
mod models;

use crate::config::AppConfig;
use crate::handlers::{health, redirect_to_url, shorten_url};
use axum::Router;
use axum::routing::{get, post};
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

    let db = init_db(&config).await?;

    let state = AppState { db, config };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/shorten", post(shorten_url))
        .route("/{code}", get(redirect_to_url))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    println!("Server running on http://{}", addr);

    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
