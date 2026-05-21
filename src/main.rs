use dioxus::prelude::*;

mod components;
#[cfg(feature = "server")]
mod memory;
mod models;
mod server_fns;
use components::{Blog, Home, Navbar, SkillsDashboard};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/skills")]
    SkillsDashboard {},
    #[route("/blog/:id")]
    Blog { id: i32 },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link {
            rel: "icon",
            href: FAVICON
        }
        document::Link {
            rel: "stylesheet",
            href: MAIN_CSS
        }
        document::Link {
            rel: "stylesheet",
            href: TAILWIND_CSS
        }
        Router::<Route> {}
    }
}
