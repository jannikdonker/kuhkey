# kuhkey_ext – Setup

## How it works

The popup is written in Rust with Dioxus. `wasm-pack` builds it to `web/wasm/`, and WXT (in `web/`) turns that into the extension. `main.ts` only loads the wasm, the Rust code does the rest.

- `src/lib.rs` starts Dioxus inside `<div id="main">` (in `entrypoints/popup/index.html`)
- `src/app.rs` main component and the shared state
- `src/components/` the list on the left and the detail view on the right
- `src/mock.rs` fake entries for now (no login or adding entries yet)
- the css is still in `entrypoints/popup/style.css`

Note: extensions don't allow `eval`, so `document::eval` and `document::Title` from Dioxus don't work. `asset!()` also doesn't work without `dx`, put images in `web/public/` instead and use them like `/cow.jpeg`.

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
# in kuhkey_ext/web/
npm install
npm run wasm   # = wasm-pack build --target web --out-dir web/wasm (in kuhkey_ext/)
npm run dev
```

`web/wasm/`, `node_modules/` and `.output/` are not in Git and have to be generated on every machine.

## Workflow

- Changes in `web/`: reload automatically.
- Changes to the Rust code (the whole UI): run `npm run wasm` again.
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
