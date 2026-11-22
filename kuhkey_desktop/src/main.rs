use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use dioxus::prelude::*;

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    let window = WindowBuilder::new()
        .with_title("KuhKey")
        .with_inner_size(LogicalSize::new(650.0, 650.0)) // initial size
        .with_min_inner_size(LogicalSize::new(250.0, 450.0)); // can't be resized below this

    dioxus::LaunchBuilder::desktop()
        .with_cfg(Config::new().with_window(window).with_menu(None))
        .launch(Root);
}

#[component]
fn Root() -> Element {
    rsx! {
        document::Stylesheet { href: TAILWIND_CSS }
        kuhkey_ui::App {}
    }
}
