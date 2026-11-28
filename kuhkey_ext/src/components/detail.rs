use dioxus::prelude::*;
use kuhkey_core::Entry;

use crate::app::{AppState, COW};

const MASK: &str = "🐄🐮🐄🐮🐄🐮🐄🐮";

// right side: cow or the selected entry
#[component]
pub fn Detail() -> Element {
    let state = use_context::<AppState>();

    let Some(id) = (state.selected)() else {
        return rsx! { Cow {} };
    };
    match state.entry(id) {
        Some(entry) => rsx! { EntryView { entry } },
        None => rsx! { Cow {} },
    }
}

#[component]
fn Cow() -> Element {
    rsx! {
        img { class: "detail-cow", src: COW, alt: "KuhKey cow" }
    }
}

#[component]
fn EntryView(entry: Entry) -> Element {
    let mut state = use_context::<AppState>();

    rsx! {
        div { class: "detail-content",
            div { class: "detail-header",
                h2 { class: "detail-title", "{entry.get_name()}" }
                button {
                    class: "detail-close",
                    title: "Close",
                    aria_label: "Close",
                    onclick: move |_| state.selected.set(None),
                    "✕"
                }
            }

            div { class: "field-card",
                UsernameField { username: entry.get_username().to_owned() }
                PasswordField { password: entry.get_password().to_owned() }
            }
        }
    }
}

#[component]
fn UsernameField(username: String) -> Element {
    let mut state = use_context::<AppState>();

    rsx! {
        div { class: "field",
            div { class: "field-label", "Username" }
            div {
                class: "field-value field-username",
                tabindex: "0",
                onclick: move |_| state.copy_username(&username),
                "{username}"
            }
        }
    }
}

#[component]
fn PasswordField(password: String) -> Element {
    let mut state = use_context::<AppState>();
    // only reveal the psw when the user means to (hover or tab onto it)
    let mut revealed = use_signal(|| false);

    rsx! {
        div { class: "field",
            div { class: "field-label", "Password" }
            div {
                class: "field-value field-password",
                tabindex: "0", // so it can be reached with tab
                onmouseenter: move |_| revealed.set(true),
                onmouseleave: move |_| revealed.set(false),
                onfocus: move |_| revealed.set(true),
                onblur: move |_| revealed.set(false),
                onclick: move |_| state.copy_password(&password),
                if revealed() {
                    "{password}"
                } else {
                    "{MASK}"
                }
            }
        }
    }
}
