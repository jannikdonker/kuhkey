use std::time::Duration;

mod server;
use anyhow::anyhow;
use kuhkey_core::VaultLocked;
use server::{UserAccount, start_server};

use crate::server::VaultNameIdPair;

const BASE_URL: &str = "http://localhost:8080/api/v1";

pub struct KuhKeyClient {
    pub client: reqwest::Client,
}

/// Checks if the string contains illegal characters and returns an error if thats the case
#[inline]
fn string_has_illegal_chars(input: &str) -> anyhow::Result<()> {
    if input.chars().all(|c| c.is_alphanumeric()) {
        Ok(())
    } else {
        Err(anyhow!("String contains illegal characters"))
    }
}

impl KuhKeyClient {
    /// Create a new KuhKey Client
    pub fn new() -> Self {
        let client = reqwest::Client::new();
        Self { client }
    }

    /// Private function to create the user tables once
    ///
    /// Do not run this in your code!
    ///
    /// Just for testing
    pub async fn create_user_table(&self) -> anyhow::Result<()> {
        self.client
            .post(format!("{BASE_URL}/create_tables"))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Create a user account
    ///
    /// This creates an entry in the user db as well as create an empty vault directory for the user
    pub async fn create_user_account(&self, user_name: impl Into<String>) -> anyhow::Result<()> {
        let user_name = user_name.into();
        string_has_illegal_chars(&user_name)?;

        self.client
            .post(format!("{BASE_URL}/create_user"))
            .json(&user_name)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Request a user
    pub async fn get_user(&self, user_name: impl Into<String>) -> anyhow::Result<UserAccount> {
        let user_name = user_name.into();

        string_has_illegal_chars(&user_name)?;

        let user_account: UserAccount = self
            .client
            .get(format!("{BASE_URL}/get_user"))
            .json(&user_name)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(user_account)
    }

    /// Request a vault from a user using the vault id
    pub async fn get_user_vault_with_id(
        &self,
        user_name: impl Into<String>,
        vault_id: impl Into<String>,
    ) -> anyhow::Result<VaultLocked> {
        let user_name = user_name.into();
        let vault_id = vault_id.into();

        string_has_illegal_chars(&user_name)?;

        let user_vault: Vec<u8> = self
            .client
            .get(format!("{BASE_URL}/get_user_vault_with_id"))
            .json(&(&user_name, &vault_id))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        kuhkey_core::VaultLocked::from_bytes(&user_vault)
    }

    /// Request a vault from a user using the vault name
    pub async fn get_user_vault_with_name(
        &self,
        user_name: impl Into<String>,
        vault_name: impl Into<String>,
    ) -> anyhow::Result<VaultLocked> {
        let user_name = user_name.into();
        let vault_name = vault_name.into();

        string_has_illegal_chars(&user_name)?;
        string_has_illegal_chars(&vault_name)?;

        let user_vault: Vec<u8> = self
            .client
            .get(format!("{BASE_URL}/get_user_vault_with_name"))
            .json(&(&user_name, &vault_name))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        kuhkey_core::VaultLocked::from_bytes(&user_vault)
    }

    /// Request all the vaults from a user
    pub async fn get_all_vaults_from_user(
        &self,
        user_name: impl Into<String>,
    ) -> anyhow::Result<Vec<VaultNameIdPair>> {
        let user_name = user_name.into();

        string_has_illegal_chars(&user_name)?;

        let vaults_from_user: Vec<VaultNameIdPair> = self
            .client
            .get(format!("{BASE_URL}/get_all_vaults_from_user"))
            .json(&user_name)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(vaults_from_user)
    }

    /// Delete a user
    pub async fn delete_user(&self, user_name: impl Into<String>) -> anyhow::Result<()> {
        let user_name = user_name.into();

        string_has_illegal_chars(&user_name)?;

        self.client
            .post(format!("{BASE_URL}/delete_user"))
            .json(&user_name)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Delete a vault from a user
    pub async fn delete_vault(
        &self,
        user_name: impl Into<String>,
        vault_name: impl Into<String>,
    ) -> anyhow::Result<()> {
        let user_name = user_name.into();
        let vault_name = vault_name.into();

        string_has_illegal_chars(&user_name)?;
        string_has_illegal_chars(&vault_name)?;

        self.client
            .post(format!("{BASE_URL}/delete_vault"))
            .json(&(&user_name, &vault_name))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Change the username of a user
    pub async fn change_user_name(
        &self,
        current_name: impl Into<String>,
        new_name: impl Into<String>,
    ) -> anyhow::Result<()> {
        let current_name = current_name.into();
        let new_name = new_name.into();

        string_has_illegal_chars(&current_name)?;
        string_has_illegal_chars(&new_name)?;

        self.client
            .post(format!("{BASE_URL}/change_user_name"))
            .json(&(&current_name, &new_name))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Returns true if either the client or the server needs to sync with each other
    ///
    /// Note: This function does not care which of the two is out of sync!
    pub async fn is_sync_needed(
        &self,
        user_name: impl Into<String>,
        vault: VaultLocked,
    ) -> anyhow::Result<bool> {
        let vault_header = vault.get_header();

        let sync_needed: bool = self
            .client
            .get(format!("{BASE_URL}/is_sync_needed"))
            .json(&(
                &user_name.into(),
                &vault_header.get_id().to_string(),
                &vault_header.get_modified(),
            ))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(sync_needed)
    }

    /// Synchronizes the local client vault with the server's copy.
    ///
    /// Depending on which copy is newer, the server resolves the sync as follows:
    ///
    /// - **Server is newer:** Returns the server's vault so the client can overwrite its local copy.
    /// - **Client is newer:** Overwrites the server's copy with the client's vault and returns it.
    /// - **Equal timestamps:** Both copies are already in sync; returns the vault unchanged.
    ///
    /// # Note
    /// If the vault does not exist on the server yet, the server stores the client's vault
    /// and returns it.
    pub async fn sync_vault(
        &self,
        user_name: impl Into<String>,
        vault: VaultLocked,
    ) -> anyhow::Result<VaultLocked> {
        let vault_bytes = vault.to_bytes();
        let synced_vault: Vec<u8> = self
            .client
            .post(format!("{BASE_URL}/sync_vault"))
            .json(&(&user_name.into(), &vault_bytes))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        kuhkey_core::VaultLocked::from_bytes(&synced_vault)
    }
}

// TODO: This will later be a lib.rs file, exposing just the Server struct and its's calls
//
// For now, leave this here so we can test the server and it's functions
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Spawn the server in a background task
    tokio::spawn(async {
        if let Err(e) = start_server().await {
            eprintln!("Server error: {e}");
        }
    });

    // Give the background task a tiny moment
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = KuhKeyClient::new();
    let mut buffer = String::new();

    println!("KuhKey Server Test environment");

    loop {
        println!("[1] To set up tables in the DB");
        println!("[2] To create a test account");
        println!("[3] To request a test account");
        println!("[4] To request a test account vault with ID of the vault");
        println!("[5] To request a test account vault with the name of the vault");
        println!("[6] To request all the vaults from the test account");
        println!("[7] To delete a test account and their vaults");
        println!("[8] To delete a vault from a test account");
        println!("[9] To change the username for a test account");
        println!("[q] to exit");

        // This is only here for testing so .expect() is fine
        std::io::stdin()
            .read_line(&mut buffer)
            .expect("failed to read from stdin");

        match buffer.trim() {
            "1" => client.create_user_table().await?,
            "2" => {
                println!("What should the test account be named?");
                let mut name_buffer = String::new();
                std::io::stdin().read_line(&mut name_buffer).unwrap();
                client.create_user_account(name_buffer.trim()).await?
            }
            "3" => {
                println!("Which user?");
                let mut name_buffer = String::new();
                std::io::stdin().read_line(&mut name_buffer).unwrap();

                let user = client.get_user(name_buffer.trim()).await.unwrap();
                println!("Got user: {user:?}");
            }
            "4" => {
                println!("Which user?");
                let mut name_buffer = String::new();
                std::io::stdin().read_line(&mut name_buffer).unwrap();

                println!("Enter vault ID: ");
                let mut vault_buffer = String::new();
                std::io::stdin().read_line(&mut vault_buffer).unwrap();

                let user_vault = client
                    .get_user_vault_with_id(name_buffer.trim(), vault_buffer.trim())
                    .await
                    .unwrap();
                println!("Got user vault: {}", user_vault.get_name());
            }
            "5" => {
                println!("Which user?");
                let mut name_buffer = String::new();
                std::io::stdin().read_line(&mut name_buffer).unwrap();

                println!("Enter vault name: ");
                let mut vault_buffer = String::new();
                std::io::stdin().read_line(&mut vault_buffer).unwrap();

                let user_vault = client
                    .get_user_vault_with_name(name_buffer.trim(), vault_buffer.trim())
                    .await
                    .unwrap();
                println!("Got user vault: {}", user_vault.get_name());
            }
            "6" => {
                println!("Which user?");
                let mut name_buffer = String::new();
                std::io::stdin().read_line(&mut name_buffer).unwrap();

                let all_user_vaults = client
                    .get_all_vaults_from_user(name_buffer.trim())
                    .await
                    .unwrap();
                println!("Got all user vaults: {all_user_vaults:?}");
            }
            "7" => {
                println!("Which user?");
                let mut name_buffer = String::new();
                std::io::stdin().read_line(&mut name_buffer).unwrap();
                client.delete_user(name_buffer.trim()).await?;
            }
            "8" => {
                println!("Which user?");
                let mut name_buffer = String::new();
                std::io::stdin().read_line(&mut name_buffer).unwrap();

                println!("Which vault?");
                let mut vault_buffer = String::new();
                std::io::stdin().read_line(&mut vault_buffer).unwrap();

                client
                    .delete_vault(name_buffer.trim(), vault_buffer.trim())
                    .await?;
            }
            "9" => {
                println!("Current username");
                let mut current_buffer = String::new();
                std::io::stdin().read_line(&mut current_buffer).unwrap();

                println!("New username");
                let mut new_buffer = String::new();
                std::io::stdin().read_line(&mut new_buffer).unwrap();

                client
                    .change_user_name(current_buffer.trim(), new_buffer.trim())
                    .await?;
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
