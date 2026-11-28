mod app;
mod browser;
mod components;
mod mock;

use wasm_bindgen::prelude::*;

// runs as soon as popup/main.ts loaded the wasm, mounts the ui into #main
#[wasm_bindgen(start)]
pub fn start() {
    dioxus::LaunchBuilder::web()
        .with_cfg(dioxus::web::Config::new().rootname("main"))
        .launch(app::App);
}
