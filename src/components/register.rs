use crate::backend::register;
use dioxus::prelude::*;
#[component]
pub fn Register() -> Element {
    // Signals for input fields
    let mut reg_username = use_signal(|| String::new());
    let mut reg_password = use_signal(|| String::new());
    let mut reg_status = use_signal(|| String::new());

    rsx! {
        div {
            // display: "grid", grid_template_columns: "1fr 1fr", gap: "20px",

            // --- Register Form ---
            div { class: "auth-form",
                h2 { "Register" }
                p { color: "red", "{reg_status}" } // Display registration status
                input {
                    r#type: "text",
                    placeholder: "Username",
                    value: "{reg_username}",
                    oninput: move |evt| {
                        reg_username.set(evt.value());
                        reg_status.set(String::new());
                    },
                }
                input {
                    r#type: "password",
                    placeholder: "Password",
                    value: "{reg_password}",
                    oninput: move |evt| {
                        reg_password.set(evt.value());
                        reg_status.set(String::new());
                    },
                }
                button {
                    onclick: move |_| {
                        let username = reg_username.read().clone();
                        let password = reg_password.read().clone();
                        if username.is_empty() || password.is_empty() {
                            reg_status.set("Username and password are required.".to_string());
                            return;
                        }
                        reg_status.set("Registering...".to_string());
                        spawn(async move {
                            match register(username.clone(), password).await {
                                Ok(_) => {
                                    reg_status
                                        .set(format!("Registered successfully! You can now login."));
                                    reg_username.set(String::new());
                                    reg_password.set(String::new());
                                }
                                Err(e) => {
                                    eprintln!("Registration error: {:?}", e);
                                    reg_status.set(format!("Registration failed: {}", e));
                                }
                            }
                        });
                    },
                    "Register"
                }

            }
        }
    }
}
