use dioxus::prelude::*;

use crate::models::skill_model::SkillModel;

/// A single skill card on the skills dashboard.
#[component]
pub fn SkillCard(skill: SkillModel, on_edit: EventHandler<SkillModel>) -> Element {
    let initial = skill.name.chars().next().unwrap_or('S');
    let id_short: String = skill.id.chars().take(8).collect();
    let edit_skill = skill.clone();

    rsx! {
        div {
            class: "flex flex-col gap-3 p-5 bg-white rounded-xl shadow-md border border-gray-200 hover:shadow-xl hover:-translate-y-1 transition-all",
            div { class: "flex items-center gap-3",
                div {
                    class: "w-12 h-12 rounded-lg bg-gradient-to-br from-emerald-500 to-teal-600 flex items-center justify-center text-white text-xl font-bold",
                    "{initial}"
                }
                div { class: "flex flex-col",
                    h3 { class: "text-lg font-semibold text-gray-800", "{skill.name}" }
                    span { class: "text-xs text-gray-400", "ID: {id_short}" }
                }
            }
            p { class: "text-sm text-gray-600 line-clamp-3 min-h-[3.5rem]", "{skill.description}" }
            div { class: "flex justify-end gap-2 pt-2 border-t border-gray-100",
                button {
                    class: "text-sm text-blue-600 hover:text-blue-800 font-medium",
                    onclick: move |_| on_edit.call(edit_skill.clone()),
                    "Edit"
                }
            }
        }
    }
}
