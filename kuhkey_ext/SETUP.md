# kuhkey_ext – Setup

## How it works

The popup UI is a Dioxus app written in Rust. WXT only provides the extension shell (manifest, background, content script, bundling).

```
kuhkey_core     Rust logic (crypto, vault, generator)
kuhkey_ext      Dioxus components (src/components/), mounted by #[wasm_bindgen(start)]
  ↓ wasm-pack build
web/wasm/       .wasm + JS wrapper
  ↓ main.ts only calls init()
web/            WXT project → .output/chrome-mv3/
```

- `src/lib.rs`: starts Dioxus and mounts it into `<div id="main">` in `entrypoints/popup/index.html`.
- `src/app.rs`: root component plus shared state (entries, selected entry, toast).
- `src/components/`: entry list (`list.rs`) and detail view (`detail.rs`).
- `src/mock.rs`: the test entries shown for now (no login, no adding/editing yet).
- `src/browser.rs`: clipboard.
- Styling still comes from `entrypoints/popup/style.css`; the components use the same class names.

Things to keep in mind (MV3 CSP allows no `eval`):
- Don't use `document::eval`, `document::Title` & co. from Dioxus, they need `eval`.
- Don't use `asset!()` (that only works with `dx`). Put static files in `web/public/`, they are served from the extension root (e.g. `/cow.jpeg`).

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
