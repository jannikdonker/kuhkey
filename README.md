<div align="center">

<h1>
  <img src="assets/icons/icon-light.svg" alt="KuhKey logo" width="72" align="center">
  KuhKey
</h1>

**A zero-knowledge, self-hosted password manager written in Rust.**

[![CI](https://github.com/jannikdonker/kuhkey/actions/workflows/rust.yml/badge.svg)](https://github.com/jannikdonker/kuhkey/actions/workflows/rust.yml)
![Rust 2024](https://img.shields.io/badge/rust-2024_edition-161616?logo=rust&logoColor=white)
![WebAssembly](https://img.shields.io/badge/browser-WebAssembly-161616?logo=webassembly&logoColor=white)
[![License: MIT](https://img.shields.io/badge/license-MIT-F7A8B8)](LICENSE)

[Features](#features) · [Architecture](#architecture) · [Self-Hosting](#self-hosting-the-server) · [Getting Started](#getting-started) · [License](#license)

</div>

---

KuhKey stores credentials in a single encrypted vault and gives you access to it from the terminal, the desktop and the browser. All platforms share one Rust core, so the cryptography and the vault format behave the same everywhere.

## Features

| Feature                  | Description                                                                                                           |
| ------------------------ | --------------------------------------------------------------------------------------------------------------------- |
| **Strong encryption**    | Argon2id key derivation and XChaCha20-Poly1305 authenticated encryption. Sensitive data is zeroized in memory after use. |
| **Password generator**   | Built on a cryptographically secure ChaCha RNG. Set the length and character classes and leave out look-alike characters. |
| **Clipboard integration**| Copied passwords are cleared from the clipboard after 30 seconds.                                                     |
| **Auto-lock**            | The vault locks itself after a period of inactivity.                                                                  |
| **Vault health report**  | Flags weak and reused passwords and includes a strength meter.                                                        |
| **Browser extension**    | Autofill, powered by the same core compiled to WebAssembly.                                                           |
| **Multiple interfaces**  | CLI, terminal UI and desktop app.                                                                                     |
| **Self-hosted sync**     | Your own `kuhkey_server` keeps vaults in sync across devices and only ever stores them encrypted.                     |

## Architecture

```
                    ┌──────────────────────┐
                    │     kuhkey_core      │
                    │ vault · crypto · gen │
                    └──────────┬───────────┘
        ┌──────────────┬───────┴──────┬──────────────┐
        ▼              ▼              ▼              ▼
   kuhkey_cli      kuhkey_ui    kuhkey_desktop   kuhkey_ext
   command line    terminal UI  desktop app      browser (WASM)
        │              │              │              │
        └──────────────┴───────┬──────┴──────────────┘
                               │  encrypted vaults only
                               ▼
                    ┌──────────────────────┐
                    │    kuhkey_server     │
                    │ self-hosted · sync   │
                    └──────────────────────┘
```

| Crate            | Description                                                                      |
| ---------------- | -------------------------------------------------------------------------------- |
| `kuhkey_core`    | Vault format, cryptography and password generation                               |
| `kuhkey_cli`     | Command-line interface (clap)                                                    |
| `kuhkey_ui`      | Terminal user interface                                                          |
| `kuhkey_desktop` | Native desktop application                                                       |
| `kuhkey_ext`     | Browser extension: Rust compiled to WASM (wasm-bindgen), TypeScript/WXT frontend |
| `kuhkey_server`  | Self-hosted sync server (axum, SQLite) for user accounts and encrypted vaults    |

## Self-Hosting the Server

> [!IMPORTANT]
> There is no public KuhKey server. To sync between devices, you host `kuhkey_server` yourself.

The server stores user accounts and their vaults. Encryption happens in the apps, so it only ever sees encrypted data.
## Getting Started

The project is a Cargo workspace, so pass `--bin` to choose which crate to run:

```bash
cargo run --bin kuhkey_cli -- init                  # create a new vault
cargo run --bin kuhkey_cli -- gen --len 24 --copy   # generate a password and copy it
cargo run --bin kuhkey_cli -- add github -g         # add an entry with a generated password
cargo run --bin kuhkey_cli -- get github --copy     # copy the password to the clipboard
```

To build the browser extension, see [`kuhkey_ext/SETUP.md`](kuhkey_ext/SETUP.md).

## License

Released under the [MIT License](LICENSE).
