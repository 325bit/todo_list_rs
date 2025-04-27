use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn NavBar() -> Element {
    rsx! {
        div { id: "title",
            Link { to: Route::Register,
                h1 { "Register" }
            }
            Link { to: Route::Login,
                h1 { "Login" }
            }
        }
        Outlet::<Route> {}
    }
}
