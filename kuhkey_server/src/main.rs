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

    async fn create_user_account(&self, name: impl Into<String>) -> reqwest::Result<()> {
        self.client
            .post(format!("{BASE_URL}/create_user"))
            .json(&UserAccount {
                id: uuid::Uuid::new_v4().to_string(),
                name: name.into(),
            })
            .send()
            .await?
            .error_for_status()?;

        println!("[INFO]: Created user and vault user directory");
        Ok(())
    }

    async fn create_user_vault(
        &self,
        user_name: impl Into<String>,
        vault_name: impl Into<String>,
    ) -> reqwest::Result<()> {
        let user_name = user_name.into();
        let vault_name = vault_name.into();

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

    async fn get_user(&self, user_name: impl Into<String>) -> reqwest::Result<UserAccount> {
        let user_name = user_name.into();
        let user_account: UserAccount = self
            .client
            .get(format!("{BASE_URL}/get_user"))
            .json(&user_name)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        println!("[INFO]: Successfully requested user `{}`", user_name);

        Ok(user_account)
    }

    async fn get_user_vault(
        &self,
        user_name: impl Into<String>,
        vault_name: impl Into<String>,
    ) -> reqwest::Result<UserVault> {
        let user_name = user_name.into();
        let vault_name = vault_name.into();

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
            "[INFO]: Successfully requested user vault from `{}` and vault name `{}`",
            user_name, vault_name
        );

        Ok(user_vault)
    }

    async fn get_all_vaults_from_user(
        &self,
        user_name: impl Into<String>,
    ) -> reqwest::Result<Vec<UserVault>> {
        let user_name = user_name.into();

        let vaults_from_user: Vec<UserVault> = self
            .client
            .get(format!("{BASE_URL}/get_all_vaults_from_user"))
            .json(&user_name)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        println!(
            "[INFO]: Successfully requested all vaults from user `{}`",
            user_name
        );

        Ok(vaults_from_user)
    }

    async fn delete_user(&self, user_name: impl Into<String>) -> reqwest::Result<()> {
        let user_name = user_name.into();

        self.client
            .post(format!("{BASE_URL}/delete_user"))
            .json(&user_name)
            .send()
            .await?
            .error_for_status()?;

        println!("[INFO]: Deleted user `{user_name}`");
        Ok(())
    }
}

// TODO: This will later be a lib.rs file, exposing just the Server struct and its's calls
//
// For now, leave this here so we can test the server and it's functions
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

    let mut buffer = String::new();

    println!("KuhKey Server Test environment");

    loop {
        println!("[1] To set up tables in the DB");
        println!("[2] To create a test account");
        println!("[3] To create a test vault for the test account");
        println!("[4] To request the test account");
        println!("[5] To request the test accounts vault");
        println!("[6] To request all the vaults from the test account");
        println!("[7] To delete a test account and their vaults");
        println!("[q] to exit");

        // This is only here for testing so .expect() is fine
        std::io::stdin()
            .read_line(&mut buffer)
            .expect("failed to read from stdin");

        match buffer.trim() {
            "1" => server.create_user_table().await?,
            "2" => {
                println!("What should the test account be named?");
                let mut name_buffer = String::new();
                std::io::stdin().read_line(&mut name_buffer).unwrap();
                server.create_user_account(name_buffer.trim()).await?
            }
            "3" => {
                println!("Enter the username for which to create the vault");
                let mut name_buffer = String::new();
                std::io::stdin().read_line(&mut name_buffer).unwrap();

                println!("What should the test vault be named?");
                let mut vault_buffer = String::new();
                std::io::stdin().read_line(&mut vault_buffer).unwrap();

                server
                    .create_user_vault(name_buffer.trim(), vault_buffer.trim())
                    .await?
            }
            "4" => {
                println!("Which user?");
                let mut name_buffer = String::new();
                std::io::stdin().read_line(&mut name_buffer).unwrap();

                let user = server.get_user(name_buffer.trim()).await.unwrap();
                println!("Got user: {user:?}");
            }
            "5" => {
                println!("Which user?");
                let mut name_buffer = String::new();
                std::io::stdin().read_line(&mut name_buffer).unwrap();

                println!("Which vault?");
                let mut vault_buffer = String::new();
                std::io::stdin().read_line(&mut vault_buffer).unwrap();

                let user_vault = server
                    .get_user_vault(name_buffer.trim(), vault_buffer.trim())
                    .await
                    .unwrap();
                println!("Got user vault: {user_vault:?}");
            }
            "6" => {
                println!("Which user?");
                let mut name_buffer = String::new();
                std::io::stdin().read_line(&mut name_buffer).unwrap();

                let all_user_vaults = server
                    .get_all_vaults_from_user(name_buffer.trim())
                    .await
                    .unwrap();
                println!("Got all user vaults: {all_user_vaults:?}");
            }
            "7" => {
                println!("Which user?");
                let mut name_buffer = String::new();
                std::io::stdin().read_line(&mut name_buffer).unwrap();
                server.delete_user(name_buffer.trim()).await?;
            }
            "q" => break,
            _ => {
                println!("Unknown command");
            }
        }

        println!("\n\n");
        buffer.clear();
    }
    Ok(())
}
