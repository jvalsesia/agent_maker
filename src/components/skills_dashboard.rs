use dioxus::prelude::*;

use crate::components::{Button, EditSkillModal, Heading, NewSkillModal, SkillCard};
use crate::models::skill_model::SkillModel;
use crate::server_fns::{create_skill, delete_skill, list_skills, update_skill};

/// Dashboard listing reusable skills as cards.
///
/// Skills are plain-text instruction bundles that can be attached to agents;
/// their text is concatenated into the agent's preamble at chat time inside
/// [`crate::server_fns::chat_with_llm`].
#[component]
pub fn SkillsDashboard() -> Element {
    let mut skills = use_signal(Vec::<SkillModel>::new);
    let mut loaded = use_signal(|| false);
    let mut show_new_modal = use_signal(|| false);
    let mut editing = use_signal(|| None::<SkillModel>);
    let mut error = use_signal(|| None::<String>);

    let loader = use_server_future(list_skills)?;

    use_effect(move || {
        if !loaded() {
            match loader.value().read().as_ref() {
                Some(Ok(list)) => {
                    skills.set(list.clone());
                    loaded.set(true);
                }
                Some(Err(e)) => {
                    error.set(Some(format!("Failed to load skills: {e}")));
                    loaded.set(true);
                }
                None => {}
            }
        }
    });

    rsx! {
        div { class: "flex flex-col w-full min-h-screen p-6 gap-6",
            div { class: "flex items-center justify-between",
                Heading { "Skills Dashboard" }
                Button {
                    onclick: move |_| show_new_modal.set(true),
                    "+ New Skill"
                }
            }

            if let Some(msg) = error() {
                p { class: "text-sm text-red-600", "{msg}" }
            }

            if skills().is_empty() {
                div { class: "flex flex-col items-center justify-center py-20 text-center gap-2",
                    p { class: "text-gray-500", "No skills yet." }
                    p { class: "text-sm text-gray-400",
                        "Skills are reusable instruction bundles you can attach to agents."
                    }
                }
            } else {
                div { class: "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6",
                    for skill in skills().iter() {
                        SkillCard {
                            skill: skill.clone(),
                            on_edit: move |s: SkillModel| editing.set(Some(s)),
                        }
                    }
                }
            }

            if show_new_modal() {
                NewSkillModal {
                    on_create: move |s: SkillModel| {
                        spawn(async move {
                            match create_skill(s.name, s.description, s.instructions).await {
                                Ok(saved) => {
                                    skills.write().push(saved);
                                    show_new_modal.set(false);
                                    error.set(None);
                                }
                                Err(e) => error.set(Some(format!("Failed to create skill: {e}"))),
                            }
                        });
                    },
                    on_close: move |_| show_new_modal.set(false),
                }
            }

            if let Some(current) = editing() {
                EditSkillModal {
                    skill: current,
                    on_save: move |s: SkillModel| {
                        spawn(async move {
                            match update_skill(s.id.clone(), s.name, s.description, s.instructions).await {
                                Ok(saved) => {
                                    let mut list = skills.write();
                                    if let Some(slot) = list.iter_mut().find(|x| x.id == saved.id) {
                                        *slot = saved;
                                    }
                                    drop(list);
                                    editing.set(None);
                                    error.set(None);
                                }
                                Err(e) => error.set(Some(format!("Failed to save skill: {e}"))),
                            }
                        });
                    },
                    on_delete: move |id: String| {
                        spawn(async move {
                            match delete_skill(id.clone()).await {
                                Ok(()) => {
                                    skills.write().retain(|x| x.id != id);
                                    editing.set(None);
                                    error.set(None);
                                }
                                Err(e) => error.set(Some(format!("Failed to delete skill: {e}"))),
                            }
                        });
                    },
                    on_close: move |_| editing.set(None),
                }
            }
        }
    }
}
