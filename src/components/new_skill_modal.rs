use dioxus::prelude::*;
use uuid::Uuid;

use crate::components::{Button, ButtonVariant, Heading, HeadingLevel};
use crate::models::skill_model::SkillModel;

const INSTRUCTIONS_PLACEHOLDER: &str = "Short, imperative rules. e.g.\n\
- Always respond in Brazilian Portuguese, formal register.\n\
- Technical terms in English (API, deploy) stay in English.\n\
- If unsure, ask one clarifying question instead of guessing.";

/// Modal form for creating a new skill (plain-text instruction bundle).
///
/// The client-generated `id` and `created_ms` are placeholders — the server
/// reassigns both during persistence.
#[component]
pub fn NewSkillModal(on_create: EventHandler<SkillModel>, on_close: EventHandler<()>) -> Element {
    let mut name = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut instructions = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);

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
        on_create.call(SkillModel {
            id: Uuid::new_v4().to_string(),
            name: n,
            description: description().trim().to_string(),
            instructions: i,
            created_ms: 0,
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
                    Heading { level: HeadingLevel::H2, "New Skill" }
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
                        placeholder: "e.g. Concise Reviewer",
                        value: "{name}",
                        oninput: move |e| name.set(e.value()),
                    }
                }

                div { class: "flex flex-col gap-1",
                    label { class: "text-sm font-medium text-gray-700", "Description (optional)" }
                    input {
                        class: "border border-gray-300 rounded-lg p-2 w-full",
                        placeholder: "One-line summary shown on the skill card.",
                        value: "{description}",
                        oninput: move |e| description.set(e.value()),
                    }
                }

                div { class: "flex flex-col gap-1",
                    label { class: "text-sm font-medium text-gray-700", "Instructions" }
                    textarea {
                        class: "border border-gray-300 rounded-lg p-2 w-full font-mono text-sm",
                        style: "min-height: 24rem;",
                        placeholder: INSTRUCTIONS_PLACEHOLDER,
                        value: "{instructions}",
                        oninput: move |e| instructions.set(e.value()),
                    }
                    p { class: "text-xs text-gray-400",
                        "Concatenated into the preamble of every agent this skill is attached to."
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
                        "Create"
                    }
                }
            }
        }
    }
}
