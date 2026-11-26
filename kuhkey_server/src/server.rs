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
    time::SystemTime,
};

const BASE_VAULT_PATH: &str = "temp/vaults";
const DATABASE_PATH: &str = "temp/kuhkey_server.db";
const FILE_EXTENSION: &str = "barn";

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
pub struct VaultNameIdPair {
    pub vault_id: String,
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
    let user_id = String::from(uuid::Uuid::new_v4());
    // We have to scope the connection here so we drop the mutex on `conn` before we use .await later down
    let rows = {
        let conn = state.connection.lock().unwrap();
        conn.execute(
            "INSERT INTO user (id, name) VALUES (?1, ?2) ON CONFLICT(name) DO NOTHING;",
            (&user_id, &username),
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };
    // The username did already exist
    if rows == 0 {
        return Err(StatusCode::CONFLICT);
    }

    // Safe to await now because the Mutex has been dropped
    let result = tokio::fs::create_dir(format!("{BASE_VAULT_PATH}/{}", user_id)).await;
    match result {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(err) => {
            eprintln!("Error while creating vault directory for user: {:?}", err);
            eprintln!("Removing DB entry...");

            // When unable to create a vault dir for the user, also delete the DB entry we just created
            // This should really never fail...
            // No need to unwrap this or return an error for this because we error the next line anyway
            let conn = state.connection.lock().unwrap();
            let _ = conn.execute("DELETE FROM user WHERE name = ?1;", (&username,));

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

/// GET /api/v1/get_user_vault_with_id
async fn db_get_user_vault_with_id(
    State(state): State<AppState>,
    Json((username, vault_id)): Json<(String, String)>,
) -> Result<Json<Vec<u8>>, StatusCode> {
    let user_id = db_get_user_id(State(state), &username)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let vault_name = format!("{vault_id}.{FILE_EXTENSION}");
    let path = PathBuf::from(BASE_VAULT_PATH)
        .join(&user_id)
        .join(&vault_name);

    if !path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    let vault_bytes = tokio::fs::read(&path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(vault_bytes))
}

/// GET /api/v1/get_user_vault_with_name
async fn db_get_user_vault_with_name(
    State(state): State<AppState>,
    Json((username, vaultname)): Json<(String, String)>,
) -> Result<Json<Vec<u8>>, StatusCode> {
    let user_id = db_get_user_id(State(state), &username)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let user_vault_dir = PathBuf::from(BASE_VAULT_PATH).join(&user_id);

    // TODO: Implement tokio::fs::read_dir here (not an iterator so bigger change)
    for entry in std::fs::read_dir(user_vault_dir).map_err(|_| StatusCode::NOT_FOUND)? {
        let vault = entry.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let vault_bytes = tokio::fs::read(vault.path())
            .await
            .map_err(|_| StatusCode::NOT_FOUND)?;

        let locked_vault = kuhkey_core::VaultLocked::from_bytes(&vault_bytes)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if locked_vault.get_name() == vaultname {
            return Ok(Json(vault_bytes));
        }
    }

    Err(StatusCode::NOT_FOUND)
}

/// GET /api/v1/get_all_vaults_from_user
async fn db_get_all_vaults_from_user(
    State(state): State<AppState>,
    Json(username): Json<String>,
) -> Result<Json<Vec<VaultNameIdPair>>, StatusCode> {
    let user_id = db_get_user_id(State(state), &username)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let vault_path = PathBuf::from(BASE_VAULT_PATH).join(&user_id);

    // TODO: Implement tokio::fs::read_dir here (not an iterator so bigger change)
    let user_vaults: Vec<VaultNameIdPair> = std::fs::read_dir(&vault_path)
        .map_err(|_| StatusCode::NOT_FOUND)?
        .filter_map(|file| file.ok()) // Ignore `None` vaules
        .filter(|file| file.path().is_file()) // Only collect files
        .map(|file| file.file_name().to_string_lossy().into_owned()) // Get file names
        .map(|file_name| {
            // TODO: Figure out how to do this without unwrapping
            let path = PathBuf::from(BASE_VAULT_PATH)
                .join(&user_id)
                .join(&file_name);
            // TODO: Implement tokio::fs::read here (closure is not async so bigger change) (no idea how to await a list of closures...)
            let vault_bytes = std::fs::read(path).unwrap();
            let vault_header = kuhkey_core::VaultLocked::from_bytes(&vault_bytes).unwrap();

            VaultNameIdPair {
                vault_id: vault_header.get_id().to_string(),
                vault_name: vault_header.get_name().to_string(),
            }
        })
        .collect();

    Ok(Json(user_vaults))
}

/// POST /api/v1/delete_user
async fn db_delete_user(
    State(state): State<AppState>,
    Json(username): Json<String>,
) -> Result<StatusCode, StatusCode> {
    let user_id = db_get_user_id(State(state.clone()), &username)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let vault_path = PathBuf::from(BASE_VAULT_PATH).join(&user_id);

    tokio::fs::remove_dir_all(&vault_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let conn = state.connection.lock().unwrap();
    conn.execute("DELETE FROM user WHERE name = ?1;", (&username,))
        .unwrap();

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/delete_vault
async fn db_delete_vault(
    State(state): State<AppState>,
    Json((username, vaultname)): Json<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    let user_id = db_get_user_id(State(state), &username)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let vault_path = PathBuf::from(BASE_VAULT_PATH)
        .join(&user_id)
        .join(&vaultname);

    if !vault_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    std::fs::remove_file(&vault_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/change_user_name
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

/// GET /api/v1/is_sync_needed
async fn db_is_sync_needed(
    State(state): State<AppState>,
    Json((username, vault_id, vault_modified)): Json<(String, String, SystemTime)>,
) -> Result<Json<bool>, StatusCode> {
    let user_id = db_get_user_id(State(state), &username)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let vault_path = PathBuf::from(BASE_VAULT_PATH)
        .join(&user_id)
        .join(format!("{vault_id}.{FILE_EXTENSION}"));

    let db_vault_bytes = match tokio::fs::read(&vault_path).await {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // When the vault does not exist on the server, the client needs to sync
            return Ok(Json(true));
        }
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let db_vault = kuhkey_core::VaultLocked::from_bytes(&db_vault_bytes)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let db_vault_modified = db_vault.get_header().get_modified();

    // The modified time of the client vault and the server vault are not the same => sync
    let need_sync = db_vault_modified != vault_modified;

    Ok(Json(need_sync))
}

/// POST /api/v1/sync_vault
async fn db_sync_vault(
    State(state): State<AppState>,
    Json((username, client_vault_bytes)): Json<(String, Vec<u8>)>,
) -> Result<Json<Vec<u8>>, StatusCode> {
    let user_id = db_get_user_id(State(state), &username)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let client_vault = kuhkey_core::VaultLocked::from_bytes(&client_vault_bytes)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let client_vault_header = client_vault.get_header();

    let client_vault_path = PathBuf::from(BASE_VAULT_PATH)
        .join(&user_id)
        .join(format!("{}.{FILE_EXTENSION}", client_vault_header.get_id()));

    let server_vault_bytes = tokio::fs::read(&client_vault_path).await;

    // This holds the synced vault:
    //
    // Client synced a vault that does not exist on the server -> Save client vault to server and return it
    //
    // Server has a newer modified timestamp -> Return the vault from the server
    //
    // Client has a newer modified timestamp -> Save client vault to server and return it
    let synced_vault: Vec<u8> = match server_vault_bytes {
        Ok(server_bytes) => {
            let db_vault = kuhkey_core::VaultLocked::from_bytes(&server_bytes)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let client_vault_timestamp = client_vault_header.get_modified();
            let db_vault_timestamp = db_vault.get_header().get_modified();

            if db_vault_timestamp.lt(&client_vault_timestamp) {
                // Server is outdated
                tokio::fs::write(&client_vault_path, &client_vault_bytes)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                client_vault_bytes
            } else if db_vault_timestamp.gt(&client_vault_timestamp) {
                // Client is outdated
                server_bytes
            } else {
                // Both the client and server are in sync
                client_vault_bytes
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // Client vault does not exist on the server, so copy it from the client
            tokio::fs::write(&client_vault_path, &client_vault_bytes)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            client_vault_bytes
        }
        Err(_) => {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(Json(synced_vault))
}

/// Local function to retrieve the user id from the DB
async fn db_get_user_id(State(state): State<AppState>, username: &str) -> rusqlite::Result<String> {
    let conn = state.connection.lock().unwrap();
    let mut statement = conn.prepare("SELECT id FROM user WHERE name = ?1")?;

    statement.query_one([username], |row| {
        let id: String = row.get(0)?;

        Ok(id)
    })
}

/// Handles routing
fn app(conn: Connection) -> Router {
    let state = AppState {
        connection: Arc::new(Mutex::new(conn)),
    };
    Router::new()
        .route("/api/v1/create_tables", post(db_create_tables))
        .route("/api/v1/create_user", post(db_create_user))
        .route("/api/v1/get_user", get(db_get_user))
        .route(
            "/api/v1/get_user_vault_with_id",
            get(db_get_user_vault_with_id),
        )
        .route(
            "/api/v1/get_user_vault_with_name",
            get(db_get_user_vault_with_name),
        )
        .route(
            "/api/v1/get_all_vaults_from_user",
            get(db_get_all_vaults_from_user),
        )
        .route("/api/v1/delete_user", any(db_delete_user))
        .route("/api/v1/delete_vault", any(db_delete_vault))
        .route("/api/v1/change_user_name", post(db_change_user_name))
        .route("/api/v1/is_sync_needed", get(db_is_sync_needed))
        .route("/api/v1/sync_vault", post(db_sync_vault))
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
