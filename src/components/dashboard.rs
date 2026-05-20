use dioxus::prelude::*;

use crate::components::{AgentCard, Button, ChatWindow, Heading, NewAgentModal};
use crate::models::agent_model::AgentModel;
use crate::server_fns::{create_agent, list_agents};

/// Dashboard listing agents as desktop-style cards.
#[component]
pub fn Dashboard() -> Element {
    let mut agents = use_signal(Vec::<AgentModel>::new);
    let mut loaded = use_signal(|| false);
    let mut active_agent = use_signal(|| None::<AgentModel>);
    let mut show_new_modal = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    // Initial load — runs on the server during SSR so the first paint already
    // contains the agent list and hydration doesn't diverge.
    let loader = use_server_future(list_agents)?;

    // Copy the loaded list into our local signal once, so subsequent mutations
    // (creating new agents) can extend it without re-fetching.
    use_effect(move || {
        if !loaded() {
            match loader.value().read().as_ref() {
                Some(Ok(list)) => {
                    agents.set(list.clone());
                    loaded.set(true);
                }
                Some(Err(e)) => {
                    error.set(Some(format!("Failed to load agents: {e}")));
                    loaded.set(true);
                }
                None => {}
            }
        }
    });

    if let Some(a) = active_agent() {
        return rsx! {
            ChatWindow {
                agent: a,
                on_close: move |_| active_agent.set(None),
            }
        };
    }

    rsx! {
        div { class: "flex flex-col w-full min-h-screen p-6 gap-6",
            div { class: "flex items-center justify-between",
                Heading { "Agents Dashboard" }
                Button {
                    onclick: move |_| show_new_modal.set(true),
                    "+ New Agent"
                }
            }

            if let Some(msg) = error() {
                p { class: "text-sm text-red-600", "{msg}" }
            }

            div { class: "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6",
                for agent in agents().iter() {
                    AgentCard {
                        agent: agent.clone(),
                        on_open: move |a: AgentModel| active_agent.set(Some(a)),
                    }
                }
            }

            if show_new_modal() {
                NewAgentModal {
                    on_create: move |a: AgentModel| {
                        // EventHandler is sync; spawn the async persistence task.
                        spawn(async move {
                            match create_agent(a.name, a.preamble, a.prompt).await {
                                Ok(saved) => {
                                    agents.write().push(saved);
                                    show_new_modal.set(false);
                                    error.set(None);
                                }
                                Err(e) => error.set(Some(format!("Failed to create agent: {e}"))),
                            }
                        });
                    },
                    on_close: move |_| show_new_modal.set(false),
                }
            }
        }
    }
}
