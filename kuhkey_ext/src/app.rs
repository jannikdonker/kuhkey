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

// stuff all components need, they get it with use_context
#[derive(Clone, Copy)]
pub struct AppState {
    pub entries: Signal<Vec<Entry>>,
    pub selected: Signal<Option<Uuid>>,
    pub notify_text: Signal<Option<String>>,
    // counts up every time, so an old timer knows a newer one came after it
    notify_count: Signal<u32>,
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

    // shows the green box for 1 sec
    pub fn notify(&mut self, text: &str) {
        self.notify_text.set(Some(text.to_string()));
        self.notify_count += 1;
        let count_now = *self.notify_count.read();

        let mut state = *self;
        // spawn_forever so the timer keeps running even if the component is gone
        spawn_forever(async move {
            TimeoutFuture::new(1_000).await;
            // only hide it if no new message came in the meantime
            if *state.notify_count.read() == count_now {
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
        let count_now = *self.copy_count.read();

        let state = *self;
        spawn_forever(async move {
            TimeoutFuture::new(30_000).await;
            // if another psw was copied in the meantime, its own timer clears it
            if *state.copy_count.read() == count_now {
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

    match (state.notify_text)() {
        Some(text) => rsx! {
            div { class: "copy-notify",
                h2 { "{text}" }
            }
        },
        None => rsx! {},
    }
}
