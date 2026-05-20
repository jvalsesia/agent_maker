use dioxus::prelude::*;
use uuid::Uuid;

use crate::components::{Button, ButtonVariant, Heading, HeadingLevel};
use crate::models::agent_model::AgentModel;

/// Modal dialog with a form to create a new agent.
///
/// Fields: **Name** (required), **Preamble** (required) — the system prompt
/// injected into every chat — and **Default prompt** (optional, currently a
/// hint for future starter-prompt UI).
///
/// The dialog closes via the ✕ button, the **Cancel** button, or by clicking
/// the backdrop. Clicks inside the dialog itself do not dismiss it.
///
/// Callbacks:
/// - `on_create`: fires with a freshly-built [`AgentModel`] (client-side
///   [`Uuid`] is generated only as a placeholder — the dashboard discards it
///   and uses the id returned by the server after persistence).
/// - `on_close`: fires whenever the user dismisses the modal without
///   submitting.
#[component]
pub fn NewAgentModal(on_create: EventHandler<AgentModel>, on_close: EventHandler<()>) -> Element {
    let mut name = use_signal(String::new);
    let mut preamble = use_signal(String::new);
    let mut prompt = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);

    let submit = move |_| {
        let n = name().trim().to_string();
        let p = preamble().trim().to_string();
        if n.is_empty() {
            error.set(Some("Name is required.".into()));
            return;
        }
        if p.is_empty() {
            error.set(Some("Preamble is required.".into()));
            return;
        }
        let agent = AgentModel {
            id: Uuid::new_v4().to_string(),
            name: n,
            preamble: p,
            prompt: prompt().trim().to_string(),
            response: String::new(),
        };
        on_create.call(agent);
    };

    rsx! {
        div {
            class: "fixed inset-0 z-50 bg-black/50 flex items-center justify-center p-4",
            onclick: move |_| on_close.call(()),
            div {
                class: "bg-white rounded-xl shadow-2xl w-full max-w-md p-6 flex flex-col gap-4",
                onclick: move |e| e.stop_propagation(),

                div { class: "flex items-center justify-between",
                    Heading { level: HeadingLevel::H2, "New Agent" }
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
                        placeholder: "e.g. Research Assistant",
                        value: "{name}",
                        oninput: move |e| name.set(e.value()),
                    }
                }

                div { class: "flex flex-col gap-1",
                    label { class: "text-sm font-medium text-gray-700", "Preamble" }
                    textarea {
                        class: "border border-gray-300 rounded-lg p-2 w-full min-h-24",
                        placeholder: "You are a helpful assistant that...",
                        value: "{preamble}",
                        oninput: move |e| preamble.set(e.value()),
                    }
                }

                div { class: "flex flex-col gap-1",
                    label { class: "text-sm font-medium text-gray-700", "Default prompt (optional)" }
                    textarea {
                        class: "border border-gray-300 rounded-lg p-2 w-full min-h-16",
                        placeholder: "Starter prompt suggestion...",
                        value: "{prompt}",
                        oninput: move |e| prompt.set(e.value()),
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
