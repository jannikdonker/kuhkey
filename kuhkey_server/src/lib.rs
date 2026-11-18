use axum::{Json, Router, extract::State, routing::post};
use reqwest::StatusCode;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

const BASE_VAULT_PATH: &str = "temp/vaults";
const DATABASE_PATH: &str = "temp/kuhkey_server.db";

#[derive(Clone)]
struct AppState {
    connection: Arc<Mutex<Connection>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserAccount {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserVault {
    pub user_name: String,
    pub vault_name: String,
}

/// POST /api/v1/create_tables
async fn db_create_tables(State(state): State<AppState>) -> StatusCode {
    let conn = state.connection.lock().unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS user (
            id    TEXT PRIMARY KEY,
            name  TEXT NOT NULL UNIQUE
        )",
        (),
    )
    // Terrible
    .unwrap();

    StatusCode::CREATED
}

/// POST /api/v1/create_user
async fn db_create_user(
    State(state): State<AppState>,
    Json(user_account): Json<UserAccount>,
) -> StatusCode {
    // Unwrap here is safe because we are not in a multithreaded environment
    let conn = state.connection.lock().unwrap();
    let rows = conn
        .execute(
            // TODO: This ignores users with the same name so we don't have multiple entries for the same user
            "INSERT INTO user (id, name) VALUES (?1, ?2) ON CONFLICT(name) DO NOTHING;",
            (&user_account.id, &user_account.name),
        )
        .unwrap();

    // This is only needed for our tests right now
    // In case we create the same user multiple times, do not create a new vault for them
    if rows == 0 {
        return StatusCode::OK;
    }

    // Create a vault directory for the user
    // This is just how we have to do it right now..
    let result = std::fs::create_dir(format!("{BASE_VAULT_PATH}/{}", user_account.id));
    match result {
        Ok(_) => StatusCode::CREATED,
        Err(err) => {
            eprintln!("Error while creating vault directory for user: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn db_create_user_vault(
    State(state): State<AppState>,
    Json(user_vault): Json<UserVault>,
) -> StatusCode {
    let Ok(user_id) = db_get_user_id(State(state), &user_vault.user_name).await else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    let result = std::fs::File::create(format!(
        "{BASE_VAULT_PATH}/{}/{}",
        &user_id, user_vault.vault_name
    ));
    match result {
        Ok(_) => StatusCode::CREATED,
        Err(err) => {
            eprintln!("Error while creating vault for user: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn db_get_user_id(
    State(state): State<AppState>,
    user_name: &str,
) -> rusqlite::Result<String> {
    let conn = state.connection.lock().unwrap();
    let mut statement = conn.prepare(&format!("SELECT id FROM user WHERE name = '{user_name}'"))?;

    let user_id = statement.query_one([], |row| {
        let id: String = row.get(0)?;

        Ok(id)
    });

    user_id
}

fn app(conn: Connection) -> Router {
    let state = AppState {
        connection: Arc::new(Mutex::new(conn)),
    };
    Router::new()
        .route("/api/v1/create_tables", post(db_create_tables))
        .route("/api/v1/create_user", post(db_create_user))
        .route("/api/v1/create_user_vault", post(db_create_user_vault))
        .with_state(state)
}

pub async fn start_server() -> rusqlite::Result<()> {
    let connection = Connection::open(DATABASE_PATH)?;

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
