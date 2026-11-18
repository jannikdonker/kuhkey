# Temporary Data

All the temporary files such as the local test database and or user vaults are saved in the kuhkey/temp directory

# Setup

Create a directory `temp` in the root directory of kuhkey

Create a `vaults` directory inside of `temp`

# API Calls

These API calls are exposed:

```rust
create_user_account(name: String);
create_user_vault(username: String, vaultname: String)
```
