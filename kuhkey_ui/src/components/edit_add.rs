use dioxus::prelude::*;
use kuhkey_core::Entry;

#[component]
pub fn EditAdd(
    entry: Entry,
    on_save: EventHandler<Entry>,
    on_cancel: EventHandler<()>,
    //TODO: new: bool, (differentiate between add and edit)
) -> Element {
    let mut name = use_signal(|| entry.name.clone());
    let mut username = use_signal(|| entry.username.clone());
    let mut password = use_signal(|| entry.password.clone());
    let mut url = use_signal(|| entry.url.clone());
    let mut notes = use_signal(|| entry.notes.clone());

    //TODO
    rsx! {
      div { class: "min-h-screen p-4 flex flex-col gap-3 bg-white dark:bg-slate-800 text-black dark:text-white",
            input { value: "{name}", oninput: move |e| name.set(e.value()) }
            input { value: "{url}", oninput: move |e| url.set(e.value()) }
            input { value: "{username}", oninput: move |e| username.set(e.value()) }
            input { r#type: "password", value: "{password}", oninput: move |e| password.set(e.value()) }
            textarea { value: "{notes}", oninput: move |e| notes.set(e.value()) }

            div { class: "flex gap-3",
                button {
                    onclick: move |_| {
                        //TODO: Save changes
                        on_save.call(entry.clone());
                    },
                    "Save"
                }
                button {
                    onclick: move |_| {
                        //TODO: Delete password placeholder incase of new password
                        on_cancel.call(());
                    },
                    "Cancel"
                }
            }
        }
    }
}
