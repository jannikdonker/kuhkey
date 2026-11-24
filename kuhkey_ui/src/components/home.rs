use super::edit_add::EditAdd;
use dioxus::prelude::*;
use kuhkey_core::Entry;

// TODO: replace with entries from kuhkey_core::open_vault
fn mock_entries() -> Vec<Entry> {
    let e = |url: &str, username: &str, name: &str, password: &str, notes: &str| {
        Entry::new(name, password, username, url, notes)
    };

    vec![
        e(
            "github.com",
            "flo2505",
            "github",
            "hunter2-hunter2",
            "Personal account",
        ),
        e(
            "users.rust-lang.org",
            "kyara",
            "rust",
            "correct-horse-battery",
            "",
        ),
        e(
            "uni.example.edu",
            "s1234567",
            "uni",
            "s3cret!pass",
            "Student portal",
        ),
    ]
}

fn copy_to_clipboard(text: &str) {
    //TODO;
}

#[component]
pub fn Home() -> Element {
    let mut entries = use_signal(mock_entries);
    let mut search = use_signal(String::new);
    let mut selected = use_signal(|| 0usize);
    let mut show_password = use_signal(|| false);
    let mut editing = use_signal(|| None::<usize>);

    let query = search().to_lowercase();
    let filtered: Vec<(usize, Entry)> = entries()
        .into_iter()
        .enumerate()
        .filter(|(_, e)| {
            query.is_empty()
                || e.get_url().to_lowercase().contains(&query)
                || e.get_name().to_lowercase().contains(&query)
                || e.get_username().to_lowercase().contains(&query)
        })
        .collect();

    let current = entries().get(selected()).cloned();

    if let Some(idx) = editing() {
        if let Some(entry) = entries().get(idx).cloned() {
            return rsx! {
                EditAdd {
                    entry,
                    //TODO: new, (differentiate between add and edit)
                    on_save: move |updated: Entry| {
                        entries.write()[idx] = updated;
                        editing.set(None);
                    },
                    on_cancel: move |_| editing.set(None),
                }
            };
        }
    }

    rsx! {
        div { class: "min-h-screen bg-white dark:bg-slate-800 text-black dark:text-white flex gap-4 p-4",
            //left: search & entry list
            aside { class: "w-64 shrink-0 flex flex-col gap-3 border border-black dark:border-white rounded-xl p-3",
                //search
                input {
                    class: "w-full px-3 py-2 bg-transparent border border-black dark:border-white rounded outline-none focus:border-green-500",
                    r#type: "search",
                    placeholder: "🔍 Search",
                    value: "{search}",
                    oninput: move |e| search.set(e.value()),
                }
                //entry list
                div { class: "flex flex-col gap-2 overflow-y-auto",
                    for (idx, entry) in filtered {
                        button {
                            key: "{idx}",
                            class: if idx == selected() {
                                "text-left px-3 py-2 border border-green-500 rounded-lg"
                            } else {
                                "text-left px-3 py-2 border border-black dark:border-white rounded-lg hover:border-green-500"
                            },
                            onclick: move |_| {
                                selected.set(idx);
                                show_password.set(false);
                            },
                            div { class: "font-semibold truncate", "{entry.get_name()}" }
                            div { class: "text-sm opacity-70 truncate", "{entry.get_username()}" }
                        }
                    }
                }
            }

            //right: detail view
            main { class: "flex-1 flex flex-col gap-5",
                if let Some(entry) = current {
                    //header: logo & name & website
                    div { class: "flex items-center gap-5",
                        //TODO: logo
                        div { class: "w-20 h-20 border-2 border-black dark:border-white rounded-xl flex items-center justify-center",
                            "logo"
                        }
                        div {
                            h1 { class: "text-2xl font-bold", "{entry.get_name()}" }
                            h2 { class: "text-xl font-semibold", "{entry.get_url()}" }
                        }

                        //edit button
                        button {
                            class: "ml-auto self-start px-2 py-1 border border-black dark:border-white rounded hover:border-green-500",
                            onclick: move |_| editing.set(Some(selected())),
                            "edit"
                        }
                    }

                    //username
                    div { class: "flex items-center gap-3",
                        div { class: "flex-1 px-3 py-2 border-b border-black dark:border-white", "{entry.get_username()}" }
                        {
                            let username = entry.get_username().to_owned();
                            rsx! {
                                button {
                                    class: "px-2 py-1 border border-black dark:border-white rounded hover:border-green-500",
                                    onclick: move |_| copy_to_clipboard(&username),
                                    "⧉"
                                }
                            }
                        }
                    }

                    //password
                    div { class: "flex items-center gap-3",
                        div { class: "flex-1 px-3 py-2 border-b border-black dark:border-white",
                            if show_password() {
                                "{entry.get_password()}"
                            } else {
                                "••••••••••"
                            }
                        }
                        button {
                            class: "px-2 py-1 border border-black dark:border-white rounded hover:border-green-500",
                            onclick: move |_| show_password.toggle(),
                            //TODO: Icon for hide and show
                            if show_password() { "hide" } else { "show" }
                        }
                        {
                            let password = entry.get_password().to_owned();
                            rsx! {
                                button {
                                    class: "px-2 py-1 border border-black dark:border-white rounded hover:border-green-500",
                                    onclick: move |_| copy_to_clipboard(&password),
                                    "⧉"
                                }
                            }
                        }
                    }

                    //notes
                    div { class: "flex flex-col gap-1",
                        label { class: "text-sm opacity-70", "Notes" }
                        div { class: "min-h-24 px-3 py-2 border border-black dark:border-white rounded-xl whitespace-pre-wrap",
                            "{entry.get_notes()}"
                        }
                    }
                } else {
                    div { class: "opacity-70", "No entry selected" }
                }
            }
        }
    }
}
