use dioxus::prelude::*;

use crate::components::{Button, ButtonVariant, Heading, HeadingLevel};
use crate::models::skill_model::SkillModel;

/// Modal for editing or deleting an existing skill. Fields are prefilled from
/// `skill`; `id` and `created_ms` are preserved verbatim.
///
/// Deletion is gated behind a one-step in-modal confirmation: the first click
/// flips the Delete button to a "Confirm delete" state; the second click
/// fires `on_delete`.
#[component]
pub fn EditSkillModal(
    skill: SkillModel,
    on_save: EventHandler<SkillModel>,
    on_delete: EventHandler<String>,
    on_close: EventHandler<()>,
) -> Element {
    let mut name = use_signal(|| skill.name.clone());
    let mut description = use_signal(|| skill.description.clone());
    let mut instructions = use_signal(|| skill.instructions.clone());
    let mut error = use_signal(|| None::<String>);
    let mut confirm_delete = use_signal(|| false);

    let skill_id = skill.id.clone();
    let skill_created = skill.created_ms;
    let delete_id = skill.id.clone();

    let submit = move |_| {
        let n = name().trim().to_string();
        let i = instructions().trim().to_string();
        if n.is_empty() {
            error.set(Some("Name is required.".into()));
            return;
        }
        if i.is_empty() {
            error.set(Some("Instructions are required.".into()));
            return;
        }
        on_save.call(SkillModel {
            id: skill_id.clone(),
            name: n,
            description: description().trim().to_string(),
            instructions: i,
            created_ms: skill_created,
        });
    };

    rsx! {
        div {
            class: "fixed inset-0 z-50 bg-black/50 flex items-center justify-center p-4",
            onclick: move |_| on_close.call(()),
            div {
                class: "bg-white rounded-xl shadow-2xl w-full p-6 flex flex-col gap-4",
                style: "max-width: 48rem;",
                onclick: move |e| e.stop_propagation(),

                div { class: "flex items-center justify-between",
                    Heading { level: HeadingLevel::H2, "Edit Skill" }
                    Button {
                        variant: ButtonVariant::Ghost,
                        onclick: move |_| on_close.call(()),
                        "✕"
                    }
                }

                div { class: "flex flex-col gap-1",
                    label { class: "text-sm font-medium text-gray-700", "Name" }
                    input {
                        class: "border border-gray-300 rounded-lg p-2 w-full",
                        value: "{name}",
                        oninput: move |e| name.set(e.value()),
                    }
                }

                div { class: "flex flex-col gap-1",
                    label { class: "text-sm font-medium text-gray-700", "Description (optional)" }
                    input {
                        class: "border border-gray-300 rounded-lg p-2 w-full",
                        value: "{description}",
                        oninput: move |e| description.set(e.value()),
                    }
                }

                div { class: "flex flex-col gap-1",
                    label { class: "text-sm font-medium text-gray-700", "Instructions" }
                    textarea {
                        class: "border border-gray-300 rounded-lg p-2 w-full font-mono text-sm",
                        style: "min-height: 24rem;",
                        value: "{instructions}",
                        oninput: move |e| instructions.set(e.value()),
                    }
                }

                if let Some(msg) = error() {
                    p { class: "text-sm text-red-600", "{msg}" }
                }

                div { class: "flex justify-between items-center pt-2 border-t border-gray-100",
                    if confirm_delete() {
                        div { class: "flex gap-2",
                            button {
                                class: "text-sm px-3 py-1.5 rounded-lg bg-red-600 text-white hover:bg-red-700",
                                onclick: move |_| on_delete.call(delete_id.clone()),
                                "Confirm delete"
                            }
                            button {
                                class: "text-sm px-3 py-1.5 rounded-lg text-gray-600 hover:text-gray-800",
                                onclick: move |_| confirm_delete.set(false),
                                "Cancel"
                            }
                        }
                    } else {
                        button {
                            class: "text-sm px-3 py-1.5 rounded-lg text-red-600 hover:bg-red-50",
                            onclick: move |_| confirm_delete.set(true),
                            "Delete"
                        }
                    }
                    div { class: "flex gap-2",
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
}
