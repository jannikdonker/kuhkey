# kuhkey_ext – Setup

## How it works

```
kuhkey_core     Rust logic (crypto, vault, generator)
kuhkey_ext      exports functions with #[wasm_bindgen]
  ↓ wasm-pack build
web/wasm/       .wasm + JS wrapper + .d.ts
  ↓ import in TypeScript
web/            WXT project → .output/chrome-mv3/
```

Rust function in `kuhkey_ext/src/lib.rs`:

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello {name}")
}
```

Called from `web/entrypoints/popup/main.ts`. `init()` loads the WASM module and must finish before the first call:

```ts
import init, { greet } from '../../wasm/kuhkey_ext.js';

init().then(() => console.log(greet('Yoshi')));
```

## First-time setup

Prerequisites: Rust ([rustup.rs](https://rustup.rs), on Windows with the C++ Build Tools) and Node.js ≥ 18.

Once per machine:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

Then reopen your terminal.

Build the project:

```bash
# in kuhkey_ext/
wasm-pack build --target web --out-dir web/wasm

# in kuhkey_ext/web/
npm install
npm run dev
```

`web/wasm/`, `node_modules/` and `.output/` are not in Git and have to be generated on every machine.

## Workflow

- Changes in `web/`: reload automatically.
- Changes to the Rust code: run `wasm-pack build` again.
- `npm run build` creates the finished extension in `web/.output/chrome-mv3/`; load it via `chrome://extensions` → "Load unpacked".

## Troubleshooting

- **`can't find library`**: `src/lib.rs` is missing.
- **`command not found` / `not recognized`**: tool not installed, or terminal not reopened.
- **`Could not read package.json`**: `npm` run in the wrong folder; it must run in `web/`.
- **`Permission denied` on `node_modules/.bin/wxt`**: cloud-sync folder. Delete `node_modules`, run `npm install`, and preferably keep the repo outside of Nextcloud/OneDrive.
- **CSP error when loading the WASM**: add this under `manifest` in `wxt.config.ts`:
  ```ts
  content_security_policy: {
    extension_pages: "script-src 'self' 'wasm-unsafe-eval'; object-src 'self';",
  },
  ```
