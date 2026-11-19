use std::time::Duration;

mod server;
use server::{UserAccount, UserVault, start_server};

const BASE_URL: &str = "http://localhost:8080/api/v1";

struct Server<'a> {
    client: &'a reqwest::Client,
}

impl<'a> Server<'a> {
    async fn create_user_table(&self) -> reqwest::Result<()> {
        self.client
            .post(format!("{BASE_URL}/create_tables"))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    async fn create_user_account(&self, name: String) -> reqwest::Result<()> {
        self.client
            .post(format!("{BASE_URL}/create_user"))
            .json(&UserAccount {
                id: uuid::Uuid::new_v4().to_string(),
                name,
            })
            .send()
            .await?
            .error_for_status()?;

        println!("[INFO]: Created user and vault user directory");
        Ok(())
    }

    async fn create_user_vault(
        &self,
        user_name: String,
        vault_name: String,
    ) -> reqwest::Result<()> {
        self.client
            .post(format!("{BASE_URL}/create_user_vault"))
            .json(&UserVault {
                user_name: user_name.clone(),
                vault_name: vault_name.clone(),
            })
            .send()
            .await?
            .error_for_status()?;

        println!(
            "[INFO]: Created a vault with name `{}` for user `{}`",
            vault_name, user_name
        );
        Ok(())
    }

    async fn get_user(&self, user_name: String) -> reqwest::Result<UserAccount> {
        let user_account: UserAccount = self
            .client
            .get(format!("{BASE_URL}/get_user"))
            .json(&user_name)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        println!("[INFO]: Successfully request user `{}`", user_name);

        Ok(user_account)
    }

    async fn get_user_vault(
        &self,
        user_name: String,
        vault_name: String,
    ) -> reqwest::Result<UserVault> {
        let user_vault: UserVault = self
            .client
            .get(format!("{BASE_URL}/get_user_vault"))
            .json(&(&user_name, &vault_name))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        println!(
            "[INFO]: Successfully request user vault from `{}` and vault name `{}`",
            user_name, vault_name
        );

        Ok(user_vault)
    }
}

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
    let server = Server { client: &client };

    server.create_user_table().await?;
    server.create_user_account(String::from("Marwin")).await?;
    server
        .create_user_vault(String::from("Marwin"), String::from("Marwin's Vault"))
        .await?;
    let user = server.get_user("Marwin".to_string()).await.unwrap();
    println!("Got user: {user:?}");

    let user_vault = server
        .get_user_vault("Marwin".to_string(), "Marwin's Vault".to_string())
        .await
        .unwrap();
    println!("Got user vault: {user_vault:?}");

    Ok(())
}
