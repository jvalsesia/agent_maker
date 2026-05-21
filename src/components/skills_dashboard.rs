use dioxus::prelude::*;

use crate::components::{Button, Heading, SkillCard};
use crate::models::skill_model::SkillModel;

/// Dashboard listing reusable skills as cards.
///
/// Skills are plain-text instruction bundles that can later be attached to
/// agents; their text is concatenated into the agent's preamble at chat time.
///
/// Persistence is not wired yet — state lives in a local signal until the
/// `list_skills` / `create_skill` server functions land.
#[component]
pub fn SkillsDashboard() -> Element {
    let skills = use_signal(Vec::<SkillModel>::new);
    let error = use_signal(|| None::<String>);

    rsx! {
        div { class: "flex flex-col w-full min-h-screen p-6 gap-6",
            div { class: "flex items-center justify-between",
                Heading { "Skills Dashboard" }
                Button {
                    onclick: move |_| {
                        // TODO: open NewSkillModal once it exists.
                    },
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
                            on_edit: move |_s: SkillModel| {
                                // TODO: open edit modal.
                            },
                        }
                    }
                }
            }
        }
    }
}
