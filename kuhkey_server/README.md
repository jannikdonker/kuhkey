i# Temporary Data

All the temporary files such as the local test database and or user vaults are saved in the kuhkey/temp directory

# Setup

Create a directory `temp` in the root directory of kuhkey

Create a `vaults` directory inside of `temp`

# API Calls

These API calls are exposed:

```rust
create_user_account(name: String) -> Result<()>;
get_user(username: String) -> Result<UserAccount>;
get_user_vault_with_id(username: String, vault_id: String) -> Result<LockedVault>;
get_user_vault_with_name(username: String, vaultname: String) -> Result<LockedVault>;
get_all_vaults_from_user(username: String) -> Result<Vec<VaultNameIdPair>>;
delete_user(name: String) -> Result<()>;
sync_needed(username: String, vault: LockedVault) -> Result<bool>;
```

Needed

```rust
sync(username: String, vault: LockedVault) -> Result<LockedVault>;
```

TODO: 

- Use non blocking tokio fs calls everywhere
