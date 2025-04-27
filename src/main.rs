use backend::{delete_todo, list_todos, save_todo};
use dioxus::prelude::*;
mod backend;

static CSS: Asset = asset!("/assets/main.css");
static BACKGROUND_IMAGE: Asset = asset!("/assets/873441.png");
fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let todos = use_resource(move || list_todos());

    use_context_provider(move || todos);

    rsx! {
        document::Stylesheet { href: CSS }
        img {
            src: BACKGROUND_IMAGE,
            style: "
                position: fixed; /* Fixed position covers the whole viewport even on scroll */
                top: 0;
                left: 0;
                width: 100%;
                height: 100%;
                object-fit: cover; /* Ensure image covers the area without distorting aspect ratio */
                z-index: -1; /* Place behind all other content */
                opacity: 0.15; /* Set the transparency */
                pointer-events: none; /* Ensure the image doesn't interfere with mouse events */
            ",
        }
        div {
            h1 { "Todo list" }

            Todo_save {}
            Todo_show {}
        }
    }
}

#[component]
pub fn Todo_save() -> Element {
    // Signal to hold the content of the input textbox
    let mut input_content = use_signal(|| String::new());
    // Signal to hold feedback message (e.g., "Saved: Buy milk")
    let mut save_status = use_signal(|| String::new());

    let todos = use_context::<Resource<Result<Vec<(usize, String)>, ServerFnError>>>();

    rsx! {
        div {
            input {
                r#type: "text",
                placeholder: "Enter a new todo...",
                value: "{input_content}", // Bind value to signal
                // Update signal whenever input changes
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
                    save_status.set("Saving...".to_string());
                    let mut todos_handle = todos.clone();
                    spawn(async move {
                        match save_todo(current_content.clone()).await {
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
                "save todo list"
            }
            p { "{save_status}" }
        }
    }
}
#[component]
pub fn Todo_show() -> Element {
    let todos = use_context::<Resource<Result<Vec<(usize, String)>, ServerFnError>>>();

    // --- Add signal for delete status ---
    let mut delete_status = use_signal(|| String::new());

    rsx! {
        div {
            h2 { "Current Todos" }
            p { "{delete_status}" }

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
                                            let mut todos_handle = todos.clone();
                                            rsx! {
                                                li { key: "{todo_id}",
                                                    "{todo_content}"
                                                    button {
                                                        onclick: move |_| {
                                                            delete_status.set("Deleting...".to_string());
                                                            spawn(async move {
                                                                match delete_todo(todo_id).await {
                                                                    Ok(_) => {
                                                                        delete_status.set(format!("Deleted todo with ID: {}", todo_id));
                                                                        todos_handle.restart();
                                                                    }
                                                                    Err(e) => {
                                                                        eprintln!("Error deleting todo: {:?}", e);
                                                                        delete_status.set(format!("Error deleting todo {}: {}", todo_id, e));
                                                                    }
                                                                }
                                                            });
                                                        },
                                                        style: "margin-left: 10px; color: red; border: none; background: none; cursor: pointer;",
                                                        "X" // The button text
                                                    }
                                                // --- End delete button ---
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
                        p { "Error loading todos: {e}" }
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
