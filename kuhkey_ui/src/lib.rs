mod components;

use components::home::Home;
use components::login::Login;
use dioxus::prelude::*;

#[component]
pub fn App() -> Element {
    let mut logged_in = use_signal(|| false);

    rsx! {
        if logged_in() {
            Home {}
        } else {
            Login { on_login: move |_| logged_in.set(true) }
        }
    }
}
