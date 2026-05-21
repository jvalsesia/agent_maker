use dioxus::prelude::*;
use std::collections::HashSet;

use crate::components::{Button, ButtonVariant, Heading, HeadingLevel};
use crate::models::agent_model::AgentModel;
use crate::models::skill_model::SkillModel;
use crate::server_fns::{list_agent_skill_ids, list_skills};

/// Modal for managing the skills attached to an existing agent.
///
/// Loads the full skill catalog and the agent's currently-attached skill IDs,
/// then on Save fires `on_save` with the new desired set. The caller diffs it
/// against the prior state and issues attach/detach calls.
#[component]
pub fn EditAgentModal(
    agent: AgentModel,
    on_save: EventHandler<(String, HashSet<String>, HashSet<String>)>,
    on_close: EventHandler<()>,
) -> Element {
    let agent_id = agent.id.clone();
    let agent_name = agent.name.clone();

    let mut skills = use_signal(Vec::<SkillModel>::new);
    let mut original = use_signal(HashSet::<String>::new);
    let mut selected = use_signal(HashSet::<String>::new);
    let mut loaded = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let load_id = agent_id.clone();
    use_future(move || {
        let aid = load_id.clone();
        async move {
            match list_skills().await {
                Ok(list) => skills.set(list),
                Err(e) => error.set(Some(format!("Failed to load skills: {e}"))),
            }
            match list_agent_skill_ids(aid).await {
                Ok(ids) => {
                    let set: HashSet<String> = ids.into_iter().collect();
                    original.set(set.clone());
                    selected.set(set);
                    loaded.set(true);
                }
                Err(e) => error.set(Some(format!("Failed to load agent skills: {e}"))),
            }
        }
    });

    let save_id = agent_id.clone();
    let submit = move |_| {
        let prev = original();
        let curr = selected();
        let to_attach: HashSet<String> = curr.difference(&prev).cloned().collect();
        let to_detach: HashSet<String> = prev.difference(&curr).cloned().collect();
        on_save.call((save_id.clone(), to_attach, to_detach));
    };

    rsx! {
        div {
            class: "fixed inset-0 z-50 bg-black/50 flex items-center justify-center p-4",
            onclick: move |_| on_close.call(()),
            div {
                class: "bg-white rounded-xl shadow-2xl w-full max-w-md p-6 flex flex-col gap-4 max-h-[90vh] overflow-y-auto",
                onclick: move |e| e.stop_propagation(),

                div { class: "flex items-center justify-between",
                    Heading { level: HeadingLevel::H2, "Edit Agent" }
                    Button {
                        variant: ButtonVariant::Ghost,
                        onclick: move |_| on_close.call(()),
                        "✕"
                    }
                }

                p { class: "text-sm text-gray-600", "Manage skills attached to ", strong { "{agent_name}" }, "." }

                div { class: "flex flex-col gap-1",
                    label { class: "text-sm font-medium text-gray-700", "Skills" }
                    if !loaded() {
                        p { class: "text-sm text-gray-400", "Loading…" }
                    } else if skills().is_empty() {
                        p { class: "text-sm text-gray-400", "No skills available yet." }
                    } else {
                        div { class: "border border-gray-300 rounded-lg p-2 max-h-72 overflow-y-auto flex flex-col gap-1",
                            for skill in skills().iter() {
                                {
                                    let sid = skill.id.clone();
                                    let sid_for_check = sid.clone();
                                    let sname = skill.name.clone();
                                    let sdesc = skill.description.clone();
                                    let is_checked = selected().contains(&sid_for_check);
                                    rsx! {
                                        label {
                                            key: "{sid}",
                                            class: "flex items-start gap-2 text-sm cursor-pointer hover:bg-gray-50 rounded px-1 py-1",
                                            input {
                                                r#type: "checkbox",
                                                class: "mt-0.5",
                                                checked: is_checked,
                                                onchange: move |e| {
                                                    let mut s = selected.write();
                                                    if e.value() == "true" {
                                                        s.insert(sid.clone());
                                                    } else {
                                                        s.remove(&sid);
                                                    }
                                                },
                                            }
                                            div { class: "flex flex-col",
                                                span { class: "font-medium text-gray-800", "{sname}" }
                                                if !sdesc.is_empty() {
                                                    span { class: "text-xs text-gray-500", "{sdesc}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(msg) = error() {
                    p { class: "text-sm text-red-600", "{msg}" }
                }

                div { class: "flex justify-end gap-2 pt-2 border-t border-gray-100",
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: submit,
                        "Save"
                    }
                }
            }
        }
    }
}
