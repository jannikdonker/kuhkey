use dioxus::prelude::*;
use kuhkey_core::Entry;

use crate::app::{AppState, COW};

// left side
#[component]
pub fn EntryList() -> Element {
    let state = use_context::<AppState>();
    let selected = (state.selected)();

    rsx! {
        nav { class: "entry-list",
            for entry in (state.entries)() {
                EntryItem {
                    key: "{entry.get_id()}",
                    selected: selected == Some(*entry.get_id()),
                    entry,
                }
            }
        }
    }
}

#[component]
fn EntryItem(entry: Entry, selected: bool) -> Element {
    let mut state = use_context::<AppState>();
    let id = *entry.get_id();
    let class = if selected {
        "entry-item selected"
    } else {
        "entry-item"
    };

    rsx! {
        button {
            class,
            onclick: move |_| state.selected.set(Some(id)),
            img { class: "entry-logo", alt: "", src: COW }
            div { class: "entry-info",
                div { class: "entry-title", "{entry.get_name()}" }
                div { class: "entry-username", "{entry.get_username()}" }
            }
        }
    }
}
