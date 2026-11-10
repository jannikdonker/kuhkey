# KuhKey

```
         .--------.
        /  .----.  \
        |  |    |  |
   =\ .-'--'----'--'-. /=
      |      ()      |
      |  o   /\   o  |
      |  .--------.  |
      | (  o    o  ) |
      |  '--------'  |
      '--------------'
```

## Feautures:

### Must have
- Encrypted vault (Argon2id + XChaCha20-Poly1305)
- Master password to unlock / lock the vault
- Add, edit and delete entries (title, username, password, url, notes)
- Password generator
- Search
- Copy to clipboard, clears itself after 30 sec
- Auto-lock after inactivity
- CLI (`kuh add`, `kuh get`, `kuh gen` ...)
- TUI
- Desktop app
- Password strength meter
- Vault health report
- Browser extension for autofill


#### Nice to have
- Breach check
- Import/export from KeePass, Bitwarden, and browser CSV
- Encrypted backups
- 2fa
- Sync between devices (server only sees encrypted data)
- Fuzzy search and tags
- Passphrase mode in the generator
- Password history, restore old passwords
- Trash for deleted entries
- Keyfile or YubiKey as second factor // YubiKey is a bit expensive
- Secure notes and encrypted attachments
- Shared vaults with friends
- German / English UI
- `kuh` without arguments prints the cow
- `kuh muh` easter egg
- Status messages with a "Muh!" here and there


# Design
- Logos: Claude Design File: @https://claude.ai/artifact/37uEgKoN16jxn6us45qMxe
