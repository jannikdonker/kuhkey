# Temporary Data

All the temporary files such as the local test database and or user vaults are saved in the kuhkey/temp directory

# Setup

Create a directory `temp` in the root directory of kuhkey

Create a `vaults` directory inside of `temp`

# API Calls

These API calls are exposed:

```rust
create_user_account(name: String) -> Result<()>;
create_user_vault(username: String, vaultname: String) -> Result<()>;
get_user(username: String) -> Result<UserAccount>;
get_user_vault(username: String) -> Result<UserVault>;
get_all_vaults_from_user(username: String) -> Result<Vec<UserVault>>;
delete_user(name: String) -> Result<()>;
```
