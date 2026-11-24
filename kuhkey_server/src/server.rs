use axum::{
    Json, Router,
    extract::State,
    routing::{any, get, post},
};
use reqwest::StatusCode;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    path::PathBuf,
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
async fn db_create_tables(State(state): State<AppState>) -> Result<StatusCode, StatusCode> {
    let conn = state.connection.lock().unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS user (
            id    TEXT PRIMARY KEY,
            name  TEXT NOT NULL UNIQUE
        )",
        (),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::CREATED)
}

/// POST /api/v1/create_user
async fn db_create_user(
    State(state): State<AppState>,
    Json(username): Json<String>,
) -> Result<StatusCode, StatusCode> {
    // Unwrap here is safe because we are not in a multithreaded environment
    let conn = state.connection.lock().unwrap();
    let user_id = String::from(uuid::Uuid::new_v4());
    let rows = conn
        .execute(
            // TODO: This ignores users with the same name so we don't have multiple entries for the same user
            "INSERT INTO user (id, name) VALUES (?1, ?2) ON CONFLICT(name) DO NOTHING;",
            (&user_id, &username),
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // This is only needed for our tests right now
    // In case we create the same user multiple times, do not create a new vault for them
    if rows == 0 {
        return Ok(StatusCode::OK);
    }

    // Create a vault directory for the user
    // This is just how we have to do it right now..
    let result = std::fs::create_dir(format!("{BASE_VAULT_PATH}/{}", &user_id));
    match result {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(err) => {
            eprintln!("Error while creating vault directory for user: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// POST /api/v1/create_user_vault
async fn db_create_user_vault(
    State(state): State<AppState>,
    Json(user_vault): Json<UserVault>,
) -> Result<StatusCode, StatusCode> {
    let Ok(user_id) = db_get_user_id(State(state), &user_vault.user_name).await else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let result = std::fs::File::create(format!(
        "{BASE_VAULT_PATH}/{}/{}",
        &user_id, user_vault.vault_name
    ));
    match result {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(err) => {
            eprintln!("Error while creating vault for user: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /api/v1/get_user
async fn db_get_user(
    State(state): State<AppState>,
    Json(username): Json<String>,
) -> Result<Json<UserAccount>, StatusCode> {
    let conn = state
        .connection
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut statement = conn
        .prepare("SELECT * FROM user WHERE name = ?1")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = statement
        .query_one([&username], |row| {
            let user_id: String = row.get(0)?;
            let username: String = row.get(1)?;

            Ok(UserAccount {
                id: user_id,
                name: username,
            })
        })
        .map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    Ok(Json(user))
}

/// GET /api/v1/get_user_vault
async fn db_get_user_vault(
    State(state): State<AppState>,
    Json((username, vaultname)): Json<(String, String)>,
) -> Result<Json<UserVault>, StatusCode> {
    let user_id = db_get_user_id(State(state), &username)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let path = PathBuf::from(BASE_VAULT_PATH)
        .join(&user_id)
        .join(&vaultname);

    if !path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    let vault_name = path
        .file_name()
        .ok_or(StatusCode::NOT_FOUND)?
        .to_string_lossy()
        .into_owned();
    Ok(Json(UserVault {
        user_name: username,
        vault_name,
    }))
}

async fn db_get_all_vaults_from_user(
    State(state): State<AppState>,
    Json(username): Json<String>,
) -> Result<Json<Vec<UserVault>>, StatusCode> {
    let user_id = db_get_user_id(State(state), &username)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let vault_path = PathBuf::from(BASE_VAULT_PATH).join(&user_id);

    let user_vaults: Vec<UserVault> = std::fs::read_dir(&vault_path)
        .map_err(|_| StatusCode::NOT_FOUND)?
        .filter_map(|file| file.ok()) // Ignore `None` vaules
        .filter(|file| file.path().is_file()) // Only collect files
        .map(|file| file.file_name().to_string_lossy().into_owned()) // Get file names
        .map(|file_name| UserVault {
            // Finally, create the user vaults
            user_name: username.clone(),
            vault_name: file_name,
        })
        .collect();

    Ok(Json(user_vaults))
}

async fn db_delete_user(
    State(state): State<AppState>,
    Json(username): Json<String>,
) -> Result<StatusCode, StatusCode> {
    let user_id = db_get_user_id(State(state.clone()), &username)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let vault_path = PathBuf::from(BASE_VAULT_PATH).join(&user_id);

    std::fs::remove_dir_all(&vault_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let conn = state.connection.lock().unwrap();
    conn.execute("DELETE FROM user WHERE name = ?1;", (&username,))
        .unwrap();

    Ok(StatusCode::NO_CONTENT)
}

async fn db_delete_vault(
    State(state): State<AppState>,
    Json((username, vaultname)): Json<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    let user_id = db_get_user_id(State(state.clone()), &username)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let vault_path = PathBuf::from(BASE_VAULT_PATH)
        .join(&user_id)
        .join(&vaultname);

    if !vault_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    std::fs::remove_file(&vault_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

async fn db_change_user_name(
    State(state): State<AppState>,
    Json((current_name, new_name)): Json<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    let conn = state.connection.lock().unwrap();

    conn.execute(
        "UPDATE user SET name = ?1 WHERE name = ?2;",
        (&new_name, &current_name),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

async fn db_get_user_id(State(state): State<AppState>, username: &str) -> rusqlite::Result<String> {
    let conn = state.connection.lock().unwrap();
    let mut statement = conn.prepare("SELECT id FROM user WHERE name = ?1")?;

    let user_id = statement.query_one([username], |row| {
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
        .route("/api/v1/get_user", get(db_get_user))
        .route("/api/v1/get_user_vault", get(db_get_user_vault))
        .route(
            "/api/v1/get_all_vaults_from_user",
            get(db_get_all_vaults_from_user),
        )
        .route("/api/v1/delete_user", any(db_delete_user))
        .route("/api/v1/delete_vault", any(db_delete_vault))
        .route("/api/v1/change_user_name", post(db_change_user_name))
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
