use dioxus::prelude::*;

use crate::models::agent_model::AgentModel;

/// A single agent card on the dashboard.
#[component]
pub fn AgentCard(
    agent: AgentModel,
    on_open: EventHandler<AgentModel>,
    on_edit: EventHandler<AgentModel>,
) -> Element {
    let initial = agent.name.chars().next().unwrap_or('A');
    let id_short: String = agent.id.chars().take(8).collect();
    let open_agent = agent.clone();
    let edit_agent = agent.clone();

    rsx! {
        div {
            class: "flex flex-col gap-3 p-5 bg-white rounded-xl shadow-md border border-gray-200 hover:shadow-xl hover:-translate-y-1 transition-all",
            div { class: "flex items-center gap-3",
                div {
                    class: "w-12 h-12 rounded-full bg-gradient-to-br from-blue-500 to-indigo-600 flex items-center justify-center text-white text-xl font-bold",
                    "{initial}"
                }
                div { class: "flex flex-col",
                    h3 { class: "text-lg font-semibold text-gray-800", "{agent.name}" }
                    span { class: "text-xs text-gray-400", "ID: {id_short}" }
                }
            }
            p { class: "text-sm text-gray-600 line-clamp-3 min-h-[3.5rem]", "{agent.preamble}" }
            div { class: "flex justify-end gap-2 pt-2 border-t border-gray-100",
                button {
                    class: "text-sm text-blue-600 hover:text-blue-800 font-medium",
                    onclick: move |_| on_open.call(open_agent.clone()),
                    "Open"
                }
                button {
                    class: "text-sm text-gray-500 hover:text-gray-700 font-medium",
                    onclick: move |_| on_edit.call(edit_agent.clone()),
                    "Edit"
                }
            }
        }
    }
}
