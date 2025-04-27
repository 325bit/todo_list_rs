use dioxus::prelude::*;

use crate::backend::login;

#[component]
pub fn Login() -> Element {
    // Signals for input fields
    let mut login_username = use_signal(|| String::new());
    let mut login_password = use_signal(|| String::new());
    let mut login_status = use_signal(|| String::new());

    // Get the logged_in_user signal setter from context
    let logged_in_user = use_context::<Signal<Option<String>>>();

    rsx! {
        div {
            // display: "grid", grid_template_columns: "1fr 1fr", gap: "20px",

            // --- Login Form ---
            div { class: "auth-form",
                h2 { "Login" }
                p { color: "red", "{login_status}" } // Display login status
                input {
                    r#type: "text",
                    placeholder: "Username",
                    value: "{login_username}",
                    oninput: move |evt| {
                        login_username.set(evt.value());
                        login_status.set(String::new());
                    },
                }
                input {
                    r#type: "password",
                    placeholder: "Password",
                    value: "{login_password}",
                    oninput: move |evt| {
                        login_password.set(evt.value());
                        login_status.set(String::new());
                    },
                }
                button {
                    onclick: move |_| {
                        let username = login_username.read().clone();
                        let password = login_password.read().clone();
                        let mut logged_in_user = logged_in_user.clone();
                        if username.is_empty() || password.is_empty() {
                            login_status.set("Username and password are required.".to_string());
                            return;
                        }
                        login_status.set("Logging in...".to_string());
                        spawn(async move {
                            match login(username.clone(), password).await {
                                Ok(_) => {
                                    login_status.set(format!("Login successful!"));
                                    logged_in_user.set(Some(username.clone()));
                                }
                                Err(e) => {
                                    eprintln!("Login error: {:?}", e);
                                    login_status.set(format!("Login failed: {}", e));
                                }
                            }
                        });
                    },
                    "Login"
                }
            }
        }
    }
}
