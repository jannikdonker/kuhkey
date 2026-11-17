use axum::{Json, Router, extract::State, routing::post};
use reqwest::StatusCode;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
struct AppState {
    connection: Arc<Mutex<Connection>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserAccount {
    pub id: usize,
    pub name: String,
}

/// POST /api/v1/create_user
async fn db_create_user(
    State(state): State<AppState>,
    Json(user_account): Json<UserAccount>,
) -> StatusCode {
    let conn = state.connection.lock().unwrap();
    conn.execute("INSERT INTO user (name) VALUES (?1)", (&user_account.name,))
        .unwrap();

    StatusCode::CREATED
}

/// POST /api/v1/create_tables
async fn db_create_tables(State(state): State<AppState>) -> StatusCode {
    let conn = state.connection.lock().unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS user (
            id    INTEGER PRIMARY KEY,
            name  TEXT NOT NULL
        )",
        (),
    )
    // Terrible
    .unwrap();

    StatusCode::CREATED
}

fn app(conn: Connection) -> Router {
    let state = AppState {
        connection: Arc::new(Mutex::new(conn)),
    };
    Router::new()
        .route("/api/v1/create_tables", post(db_create_tables))
        .route("/api/v1/create_user", post(db_create_user))
        .with_state(state)
}

pub async fn start_server() -> rusqlite::Result<()> {
    let connection = Connection::open("kuhkey_server.db")?;

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    // Bind to localhost only for now — not reachable from other machines.
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind port 8080");

    tracing::info!("KuhKey server listening on http://{addr}");
    axum::serve(listener, app(connection))
        .await
        .expect("server error");

    Ok(())
}
