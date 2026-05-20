use dioxus::prelude::*;

use crate::components::ChatComponent;
use crate::models::agent_model::AgentModel;

/// Full-area chat window that replaces the dashboard content. Responsive
/// across mobile and desktop — fills the viewport on small screens and
/// centers a comfortable column on larger ones.
#[component]
pub fn ChatWindow(agent: AgentModel, on_close: EventHandler<()>) -> Element {
    rsx! {
        div {
            class: "flex flex-col w-full min-h-screen p-2 sm:p-4 md:p-6 gap-3 sm:gap-4",
            div { class: "flex flex-wrap items-center justify-between gap-2",
                div { class: "flex flex-col min-w-0",
                    h1 { class: "text-2xl font-bold text-gray-800", "{agent.name}" }

                }
                div {
                button {
                    class: "bg-blue-600 text-white rounded-lg px-4 py-2 hover:bg-blue-700 text-sm whitespace-nowrap",
                    onclick: move |_| on_close.call(()),
                    "← Back"
                }
                }
            }
            div { class: "flex justify-center w-full",
                div { class: "w-full max-w-2xl",
                    ChatComponent {
                        agent_id: agent.id.clone(),
                        preamble: agent.preamble.clone(),
                    }
                }
            }
        }
    }
}
