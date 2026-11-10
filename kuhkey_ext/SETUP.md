# kuhkey_ext – Setup

## Funktionsweise

```
kuhkey_core     Rust-Logik (Crypto, Vault, Generator)
kuhkey_ext      exportiert Funktionen mit #[wasm_bindgen]
  ↓ wasm-pack build
web/wasm/       .wasm + JS-Wrapper + .d.ts
  ↓ import in TypeScript
web/            WXT-Projekt → .output/chrome-mv3/
```

Rust-Funktion in `kuhkey_ext/src/lib.rs`:

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hallo {name}")
}
```

Aufruf in `web/entrypoints/popup/main.ts`. `init()` lädt das WASM-Modul und muss vor dem ersten Aufruf fertig sein:

```ts
import init, { greet } from '../../wasm/kuhkey_ext.js';

init().then(() => console.log(greet('Yoshi')));
```

## Erstes Setup

Voraussetzungen: Rust ([rustup.rs](https://rustup.rs), unter Windows mit C++ Build Tools) und Node.js ≥ 18.

Einmal pro Rechner:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

Danach Terminal neu öffnen.

Projekt bauen:

```bash
# in kuhkey_ext/
wasm-pack build --target web --out-dir web/wasm

# in kuhkey_ext/web/
npm install
npm run dev
```

`web/wasm/`, `node_modules/` und `.output/` sind nicht im Git und müssen auf jedem Rechner erzeugt werden.

## Workflow

- Änderungen in `web/`: lädt automatisch neu.
- Änderungen am Rust-Code: `wasm-pack build` erneut ausführen.
- `npm run build` erzeugt die fertige Extension in `web/.output/chrome-mv3/`, laden über `chrome://extensions` → „Entpackte Erweiterung laden“.

## Fehler

- **`can't find library`**: `src/lib.rs` fehlt.
- **`command not found` / `not recognized`**: Tool nicht installiert oder Terminal nicht neu geöffnet.
- **`Could not read package.json`**: `npm` im falschen Ordner, muss in `web/` laufen.
- **`Permission denied` bei `node_modules/.bin/wxt`**: Cloud-Sync-Ordner. `node_modules` löschen, `npm install`, Repo besser außerhalb von Nextcloud/OneDrive ablegen.
- **CSP-Fehler beim Laden des WASM**: in `wxt.config.ts` unter `manifest` ergänzen:
  ```ts
  content_security_policy: {
    extension_pages: "script-src 'self' 'wasm-unsafe-eval'; object-src 'self';",
  },
  ```
