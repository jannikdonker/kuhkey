use std::time::Duration;

use kuhkey_server::{UserAccount, start_server};

const BASE_URL: &str = "http://localhost:8080/api/v1";

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    // Spawn the server in a background task
    tokio::spawn(async {
        if let Err(e) = start_server().await {
            eprintln!("Server error: {e}");
        }
    });

    // Give the background task a tiny moment
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = reqwest::Client::new();

    create_user_table(&client).await?;
    create_user_account(&client, String::from("Marwin")).await?;

    Ok(())
}

async fn create_user_table(client: &reqwest::Client) -> reqwest::Result<()> {
    client
        .post(format!("{BASE_URL}/create_tables"))
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

async fn create_user_account(client: &reqwest::Client, name: String) -> reqwest::Result<()> {
    client
        .post(format!("{BASE_URL}/create_user"))
        .json(&UserAccount {
            id: 0, // Maybe UUID?
            name,
        })
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}
