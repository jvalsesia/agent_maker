use dioxus::prelude::*;
use uuid::Uuid;

use crate::components::{AgentCard, ChatWindow};
use crate::models::agent_model::AgentModel;

/// Dashboard listing agents as desktop-style cards.
#[component]
pub fn Dashboard() -> Element {
    let agents = use_signal(|| {
        vec![
            AgentModel {
                id: Uuid::new_v4().to_string(),
                name: "General Assistant".to_string(),
                preamble: "You are a helpful assistant.".to_string(),
                prompt: "Summarize the latest research on this topic.".to_string(),
                response: String::new(),
            },
            
        ]
    });

    let mut active_agent = use_signal(|| None::<AgentModel>);

    if let Some(a) = active_agent() {
        return rsx! {
            ChatWindow {
                agent: a,
                on_close: move |_| active_agent.set(None),
            }
        };
    }

    rsx! {
        div {
            class: "flex flex-col w-full min-h-screen p-6 gap-6",
            div { class: "flex items-center justify-between",
                h1 { class: "text-2xl font-bold text-gray-800", "Agents Dashboard" }
                button {
                    class: "bg-blue-600 text-white rounded-lg px-4 py-2 hover:bg-blue-700",
                    "+ New Agent"
                }
            }

            div {
                class: "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6",
                for agent in agents().iter() {
                    AgentCard {
                        agent: agent.clone(),
                        on_open: move |a: AgentModel| active_agent.set(Some(a)),
                    }
                }
            }
        }
    }
}
