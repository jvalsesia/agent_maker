use dioxus::prelude::*;

use crate::server_fns::{chat_with_llm, load_history, ChatTurn};

type ChatMessage = ChatTurn;

#[component]
pub fn ChatComponent(agent_id: String, preamble: String) -> Element {
    let mut messages = use_signal(Vec::<ChatMessage>::new);
    let mut draft = use_signal(String::new);
    let preamble_state = use_signal(|| preamble);
    let agent_id_state = use_signal(|| agent_id);

    // Hydrate on mount with the persisted transcript from Postgres.
    let _hydrate = use_resource(move || async move {
        if let Ok(turns) = load_history(agent_id_state()).await {
            messages.set(turns);
        }
    });

    let send_draft = move |_| async move {
        let text = draft();
        if text.trim().is_empty() {
            return;
        }
        draft.set(String::new());
        messages.write().push(ChatMessage {
            role: "user".into(),
            content: text.clone(),
        });
        match chat_with_llm(agent_id_state(), preamble_state(), text).await {
            Ok(reply) => messages.write().push(ChatMessage {
                role: "assistant".into(),
                content: reply,
            }),
            Err(e) => messages.write().push(ChatMessage {
                role: "assistant".into(),
                content: format!("Error: {e}"),
            }),
        }
    };

    rsx! {
        div {
            class: "flex flex-col gap-3 p-3 sm:p-6 w-full max-w-2xl bg-white rounded-xl shadow-lg border border-gray-200 box-border overflow-hidden",
            h2 { class: "text-xl font-bold", "Chat" }

            div {
                class: "flex flex-col gap-2 border rounded p-3 min-h-64 max-h-96 overflow-y-auto",
                style: "background-color: #faf6ee;",
                for (i, msg) in messages().iter().enumerate() {
                    div {
                        key: "{i}",
                        class: if msg.role == "user" { "self-end bg-blue-100 rounded px-3 py-2 max-w-md" } else { "self-start border rounded px-3 py-2 max-w-md" },
                        style: if msg.role == "assistant" { "background-color: #ffe8d6; border-color: #fed7aa;" } else { "" },
                        span { class: "block text-xs text-gray-500", "{msg.role}" }
                        span { class: "whitespace-pre-wrap", "{msg.content}" }
                    }
                }
            }

            div { class: "flex flex-col sm:flex-row gap-2 w-full items-stretch sm:items-center",
                input {
                    class: "border rounded p-2 flex-1 min-w-0 w-full",
                    style: "background-color: #f5efe6;",
                    placeholder: "Type a message...",
                    value: "{draft}",
                    oninput: move |e| draft.set(e.value()),
                }
                button {
                    class: "bg-blue-600 text-white rounded text-sm px-3 py-1 hover:bg-blue-700 self-end shrink-0",
                    onclick: send_draft,
                    "Send"
                }
            }
        }
    }
}
