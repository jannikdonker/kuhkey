use dioxus::prelude::*;

#[component]
pub fn Login(on_login: EventHandler<()>) -> Element {
    let mut user = use_signal(String::new);
    let mut password = use_signal(String::new);

    rsx! {
        div { class: "min-h-screen bg-white dark:bg-slate-800 text-black dark:text-white flex items-center justify-center",
        form {
            class: "flex flex-col items-center gap-5 w-73 m-5",
            onsubmit: move |evt| {
                evt.prevent_default();
                //TODO: kuhkey_core
                on_login.call(());
            },

           // TODO: logo placeholder (swap for an img once you have one)
                div { class: "w-30 h-30 border-2 border-black dark:border-white rounded flex items-center justify-center",
                    "logo"
                }
                // img { src: asset!("/assets/logo.png"), class: "w-30 h-30" }

                h1 { class: "text-3xl tracking-tight",
                    span { class: "font-bold", "Kuh" }
                    span { class: "font-semibold", "Key" }
                }

                input {
                    class: "w-full px-3 py-2 bg-transparent border border-black dark:border-white rounded outline-none focus:border-green-500",
                    placeholder: "username",
                    value: "{user}",
                    oninput: move |e| user.set(e.value()),
                }
                input {
                    class: "w-full px-3 py-2 bg-transparent border border-black dark:border-white rounded outline-none focus:border-green-500",
                    r#type: "password",
                    placeholder: "password",
                    value: "{password}",
                    oninput: move |e| password.set(e.value()),
                }
                button {
                    class: "w-full py-2 bg-green-600 rounded hover:bg-green-500",
                    r#type: "submit",
                    "Login"
                }
            }
        }
    }
}
