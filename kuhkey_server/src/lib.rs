use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize)]
struct Pong {
    status: &'static str,
    version: &'static str,
}

#[derive(Deserialize, Serialize)]
struct EchoMessage {
    message: String,
}

async fn ping() -> Json<Pong> {
    Json(Pong {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// POST /api/v1/echo {"message":"hi"} -> {"message":"hi"}
async fn echo(Json(body): Json<EchoMessage>) -> Json<EchoMessage> {
    tracing::info!("echo request received");
    Json(body)
}

fn app() -> Router {
    Router::new()
        .route("/api/v1/ping", get(ping))
        .route("/api/v1/echo", post(echo))
}

pub async fn start_server() -> Result<(), std::io::Error> {
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
    axum::serve(listener, app()).await.expect("server error");

    Ok(())
}
