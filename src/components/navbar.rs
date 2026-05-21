use dioxus::prelude::*;

use crate::Route;

/// Shared navbar component.
#[component]
pub fn Navbar() -> Element {
    rsx! {
        div {
            id: "navbar",
            Link {
                to: Route::Home {},
                "Agents"
            }
            Link {
                to: Route::SkillsDashboard {},
                "Skills"
            }
        }

        Outlet::<Route> {}
    }
}
