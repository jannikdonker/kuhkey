use std::time::Duration;

use kuhkey_server::start_server;
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "http://localhost:8080/api/v1";

#[derive(Deserialize, Debug)]
struct Pong {
    status: String,
    version: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct EchoMessage {
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    // Spawn the server in a background task so it doesn't block main()
    tokio::spawn(async {
        if let Err(e) = start_server().await {
            eprintln!("Server error: {e}");
        }
    });

    // Give the background task a tiny moment to bind port 8080
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Reuse one client for all requests (it keeps a connection pool).
    let client = reqwest::Client::new();

    // GET /ping  (same as: curl localhost:8080/api/v1/ping)
    let pong: Pong = client
        .get(format!("{BASE_URL}/ping"))
        .send()
        .await?
        .error_for_status()? // turn 4xx/5xx into an Err
        .json()
        .await?;
    println!("ping -> status={}, version={}", pong.status, pong.version);

    // POST /echo  (same as: curl -X POST ... -d '{"message":"hello"}')
    // .json() serializes the struct and sets Content-Type: application/json.
    let reply: EchoMessage = client
        .post(format!("{BASE_URL}/echo"))
        .json(&EchoMessage {
            message: "hello".into(),
        })
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    println!("echo -> {}", reply.message);

    Ok(())
}
