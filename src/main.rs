mod models;
mod errors;
mod handlers;

use crate::handlers::{redirect_to_url, shorten_url};
use axum::Router;
use axum::extract::State;
use axum::routing::{get, post};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub base_url: String,
    pub code_length: usize,
}

async fn health(
    State(state): State<AppState>
) -> &'static str {
    match sqlx::query("select 1")
        .execute(&state.db)
        .await {
        Ok(_) => "OK",
        Err(_) => "Database Unavailable"
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")?;
    let base_url = std::env::var("BASE_URL")?;
    let code_length = std::env::var("CODE_LENGTH")?.trim().parse()?;

    let db = SqlitePool::connect(&database_url).await?;

    println!("Connected to sqlite");

    let state = AppState{ db, base_url, code_length };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/shorten", post(shorten_url))
        .route("/{code}", get(redirect_to_url))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    println!("Server running on http://{}", addr);

    let listener = TcpListener::bind(addr)
        .await?;

    axum::serve(listener, app)
        .await?;

    Ok(())
}
