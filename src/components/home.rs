use dioxus::prelude::*;

use super::Dashboard;

/// Home page — defaults to the agents Dashboard.
#[component]
pub fn Home() -> Element {
    rsx! {
        Dashboard {}
    }
}
