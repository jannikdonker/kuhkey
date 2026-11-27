use dioxus::core::spawn_forever;
use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use kuhkey_core::Entry;
use uuid::Uuid;

use crate::browser;
use crate::components::detail::Detail;
use crate::components::list::EntryList;
use crate::mock;

pub const COW: &str = "/cow.jpeg";

#[derive(Clone, Copy)]
pub struct AppState {
    pub entries: Signal<Vec<Entry>>,
    pub selected: Signal<Option<Uuid>>,
    pub notify_text: Signal<Option<String>>,
    pub notify_count: Signal<u32>,
    copy_count: Signal<u32>,
}

impl AppState {
    pub fn entry(&self, id: Uuid) -> Option<Entry> {
        for entry in self.entries.read().iter() {
            if *entry.get_id() == id {
                return Some(entry.clone());
            }
        }
        None
    }

    pub fn notify(&mut self, text: &str) {
        self.notify_count += 1;
        self.notify_text.set(Some(text.to_string()));

        let my_count = (self.notify_count)();
        let mut state = *self;
        spawn_forever(async move {
            TimeoutFuture::new(1_000).await;
            if (state.notify_count)() == my_count {
                state.notify_text.set(None);
            }
        });
    }

    pub fn copy_username(&mut self, username: &str) {
        browser::copy(username);
        self.notify("Username copied!");
    }

    // copies the psw and clears the clipboard again after 30 sec
    pub fn copy_password(&mut self, password: &str) {
        browser::copy(password);
        self.notify("Password copied!");

        self.copy_count += 1;
        let my_count = (self.copy_count)();
        let state = *self;
        spawn_forever(async move {
            TimeoutFuture::new(30_000).await;
            if (state.copy_count)() == my_count {
                browser::copy("");
            }
        });
    }
}

#[component]
pub fn App() -> Element {
    use_context_provider(|| AppState {
        entries: Signal::new(mock::entries()),
        selected: Signal::new(None),
        notify_text: Signal::new(None),
        notify_count: Signal::new(0),
        copy_count: Signal::new(0),
    });

    rsx! {
        header { class: "title-box",
            div { class: "title", "KuhKey" }
            div { class: "subtitle", "Your Password Manager" }
        }

        main { class: "layout",
            EntryList {}
            section { class: "detail", Detail {} }
            Notify {}
        }
    }
}

#[component]
fn Notify() -> Element {
    let state = use_context::<AppState>();
    let count = (state.notify_count)();

    let Some(text) = (state.notify_text)() else {
        return rsx! {};
    };

    rsx! {
        for count in [count] {
            div { key: "{count}", class: "copy-notify show",
                h2 { "{text}" }
            }
        }
    }
}
