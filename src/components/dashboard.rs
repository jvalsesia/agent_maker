use dioxus::prelude::*;

use crate::components::{AgentCard, Button, ChatWindow, EditAgentModal, Heading, NewAgentModal};
use crate::models::agent_model::AgentModel;
use crate::server_fns::{attach_skill, create_agent, detach_skill, list_agents};

/// Dashboard listing agents as desktop-style cards.
#[component]
pub fn Dashboard() -> Element {
    let mut agents = use_signal(Vec::<AgentModel>::new);
    let mut loaded = use_signal(|| false);
    let mut active_agent = use_signal(|| None::<AgentModel>);
    let mut show_new_modal = use_signal(|| false);
    let mut editing = use_signal(|| None::<AgentModel>);
    let mut error = use_signal(|| None::<String>);

    let loader = use_server_future(list_agents)?;

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
                        on_edit: move |a: AgentModel| editing.set(Some(a)),
                    }
                }
            }

            if show_new_modal() {
                NewAgentModal {
                    on_create: move |(a, skill_ids): (AgentModel, Vec<String>)| {
                        spawn(async move {
                            match create_agent(a.name, a.preamble, a.prompt).await {
                                Ok(saved) => {
                                    let aid = saved.id.clone();
                                    for sid in skill_ids {
                                        if let Err(e) = attach_skill(aid.clone(), sid).await {
                                            error.set(Some(format!("Failed to attach skill: {e}")));
                                        }
                                    }
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

            if let Some(current) = editing() {
                EditAgentModal {
                    agent: current,
                    on_save: move |(aid, to_attach, to_detach): (String, std::collections::HashSet<String>, std::collections::HashSet<String>)| {
                        spawn(async move {
                            for sid in to_attach {
                                if let Err(e) = attach_skill(aid.clone(), sid).await {
                                    error.set(Some(format!("Failed to attach skill: {e}")));
                                    return;
                                }
                            }
                            for sid in to_detach {
                                if let Err(e) = detach_skill(aid.clone(), sid).await {
                                    error.set(Some(format!("Failed to detach skill: {e}")));
                                    return;
                                }
                            }
                            editing.set(None);
                            error.set(None);
                        });
                    },
                    on_close: move |_| editing.set(None),
                }
            }
        }
    }
}
