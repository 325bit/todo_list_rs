use backend::{delete_todo, list_todos, save_todo};
use components::{login::Login, nav::NavBar, register::Register};
use dioxus::prelude::*;
mod backend;
mod components;
// use components::nav::NavBar;
static CSS: Asset = asset!("/assets/main.css");
static BACKGROUND_IMAGE: Asset = asset!("/assets/873441.png");

fn main() {
    dioxus::launch(App);
}
#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[layout(NavBar)]
    #[route("/")]
    Login,

    #[route("/register")]
    Register,
}

// Main Application Component
#[component]
fn App() -> Element {
    // State to hold the currently logged-in user's username. None if not logged in.
    let mut logged_in_user = use_signal(|| Option::<String>::None);

    // Resource to fetch todos. It depends on the logged_in_user state.
    let todos = use_resource(move || {
        // Read the current user state and CLONE it *before* the async block
        // This ensures the async block captures an owned value, not a reference
        // tied to the lifetime of the FnMut closure.
        let current_user_clone = logged_in_user.read().clone();
        async move {
            // Only attempt to list todos if a user is logged in
            if let Some(username) = current_user_clone {
                // Use the cloned value
                list_todos(username).await
            } else {
                // If not logged in, return an empty success state for the resource
                Ok(Vec::new())
            }
        }
    });

    // Provide the logged_in_user signal and the todos resource via context
    // This allows child components (Auth, Todo_save, Todo_show) to access them
    use_context_provider(move || logged_in_user);
    use_context_provider(move || todos);

    rsx! {
        document::Stylesheet { href: CSS }
        img {
            src: BACKGROUND_IMAGE,
            style: "
                position: fixed;
                top: 0; left: 0;
                width: 100%; height: 100%;
                object-fit: cover;
                z-index: -1;
                opacity: 0.15;
                pointer-events: none;
            ",
        }
        div { class: "app-container",

            h1 { "Todo list" }

            // Conditionally rergba(10, 10, 10, 0.8)s or Todo list based on login state
            match logged_in_user.read().as_ref() {
                Some(username) => rsx! {
                    // --- Logged-in View ---
                    p { "Logged in as: {username}" }
                    button {
                        onclick: move |_| {
                            logged_in_user.set(None);
                        },
                        "Logout"
                    }
                    hr {} // Separator

                    Todo_save {}
                    Todo_show {}
                },
                None => rsx! {
                    // --- Authentication View ---
                    Router::<Route> {}
                },
            }
        }
    }
}

// Todo Save Component (Modified)
#[component]
pub fn Todo_save() -> Element {
    let mut input_content = use_signal(|| String::new());
    let mut save_status = use_signal(|| String::new());

    // Get the logged_in_user signal and todos resource from context
    let logged_in_user = use_context::<Signal<Option<String>>>();
    let todos = use_context::<Resource<Result<Vec<(usize, String)>, ServerFnError>>>();

    rsx! {
        div {
            class: "todo-input-area",
            margin_top: "20px",
            padding_top: "20px",
            border_top: "1px solid #eee",

            input {
                r#type: "text",
                placeholder: "Enter a new todo...",
                value: "{input_content}",
                oninput: move |evt| {
                    save_status.set(String::new());
                    input_content.set(evt.value());
                },
            }
            button {
                onclick: move |_| {
                    let current_content = input_content.read().clone();
                    if current_content.trim().is_empty() {
                        save_status.set("Cannot save an empty todo.".to_string());
                        return;
                    }
                    let current_user = logged_in_user.read().clone();
                    if current_user.is_none() {
                        save_status.set("Error: Not logged in.".to_string());
                        return;
                    }
                    let username = current_user.unwrap();
                    save_status.set("Saving...".to_string());
                    let mut todos_handle = todos.clone();
                    let username_for_save = username.clone();
                    let content_for_save = current_content.clone();
                    spawn(async move {
                        match save_todo(username_for_save, content_for_save).await {
                            Ok(_) => {
                                save_status.set(format!("Saved: {}", current_content));
                                input_content.set(String::new());
                                todos_handle.restart();
                            }
                            Err(e) => {
                                eprintln!("Error saving todo: {:?}", e);
                                save_status.set(format!("Error saving: {}", e));
                            }
                        }
                    });
                },
                "Add Todo"
            }
            p { "{save_status}" }
        }
    }
}

// Todo Show Component (Modified)
#[component]
pub fn Todo_show() -> Element {
    // Get the logged_in_user signal and todos resource from context
    let logged_in_user = use_context::<Signal<Option<String>>>();
    let todos = use_context::<Resource<Result<Vec<(usize, String)>, ServerFnError>>>();

    let mut delete_status = use_signal(|| String::new());

    // Read and clone the user state ONCE for this component's rendering cycle
    // Since this component is only rendered when logged_in_user is Some,
    // we can safely unwrap.
    let current_user_clone = logged_in_user.read().clone();
    let username = current_user_clone.expect("Todo_show rendered without logged_in_user");

    rsx! {
        div {
            margin_top: "20px",
            padding_top: "20px",
            border_top: "1px solid #eee",

            h2 { "My Todos" }
            p { color: "red", "{delete_status}" } // Display delete status

            // Match on the todos resource state
            match todos.read().as_ref() {
                Some(Ok(todo_list)) => {
                    if todo_list.is_empty() {
                        rsx! {
                            p { "No todos yet!" }
                        }
                    } else {
                        rsx! {
                            ul {
                                {
                                    todo_list
                                        .iter()
                                        .map(|(id, todo_content)| {
                                            let todo_id = *id;
                                            let todo_content_clone = todo_content.clone();
                                            let username_for_delete = username.clone();
                                            let mut todos_handle = todos.clone();
                                            rsx! {
                                                li { key: "{todo_id}",
                                                    "{todo_content_clone}"
                                                    button {
                                                        onclick: move |_| {
                                                            delete_status.set(format!("Deleting todo {}...", todo_id));
                                                            let username_for_delete_clone = username_for_delete.clone();
                                                            spawn(async move {
                                                                match delete_todo(username_for_delete_clone, todo_id).await {
                                                                    Ok(_) => {
                                                                        delete_status.set(format!("Deleted todo with ID: {}", todo_id));
                                                                        todos_handle.restart();
                                                                    }
                                                                    Err(e) => {
                                                                        eprintln!("Error deleting todo {}: {:?}", todo_id, e);
                                                                        delete_status.set(format!("Error deleting todo {}: {}", todo_id, e));
                                                                    }
                                                                }
                                                            });
                                                        },
                                                        style: "margin-left: 10px; color: red; border: none; background: none; cursor: pointer;",
                                                        "X" // The button text
                                                    }
                                                }
                                            }
                                        })
                                }
                            }
                        }
                    }
                }
                Some(Err(e)) => {
                    rsx! {
                        p { color: "red", "Error loading todos: {e}" }
                    }
                }
                None => {
                    rsx! {
                        p { "Loading todos..." }
                    }
                }
            }
        }
    }
}
