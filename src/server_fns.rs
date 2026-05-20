use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use crate::memory;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatTurn {
    pub role: String,
    pub content: String,
}

/// Load this agent's full persisted chat transcript (oldest first).
#[server]
pub async fn load_history(agent_id: String) -> Result<Vec<ChatTurn>, ServerFnError> {
    let rows = memory::load_history(&agent_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows
        .into_iter()
        .map(|(role, content)| ChatTurn { role, content })
        .collect())
}

/// Chat with an OpenAI-backed LLM, using Postgres + pgvector for both verbatim
/// transcript persistence and top-k semantic recall over older turns.
#[server]
pub async fn chat_with_llm(
    agent_id: String,
    preamble: String,
    prompt: String,
) -> Result<String, ServerFnError> {
    use rig_core::client::{CompletionClient, ProviderClient};
    use rig_core::completion::{Chat, Message};
    use rig_core::providers::openai;

    // Verbatim window: last N turns sent to the model as-is.
    const RECENT_WINDOW: usize = 12;
    // Semantic recall: top-k older turns retrieved by similarity to the prompt.
    const RECALL_K: usize = 4;

    let full = memory::load_history(&agent_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let split = full.len().saturating_sub(RECENT_WINDOW);
    let (older, recent) = full.split_at(split);

    let recalled = if !older.is_empty() {
        memory::recall(&agent_id, &prompt, RECALL_K)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
    } else {
        Vec::new()
    };

    let recent_set: std::collections::HashSet<&String> =
        recent.iter().map(|(_, c)| c).collect();
    let recall_block: String = recalled
        .iter()
        .filter(|(_, c)| !recent_set.contains(c))
        .map(|(r, c)| format!("- ({r}) {c}"))
        .collect::<Vec<_>>()
        .join("\n");

    let effective_preamble = if recall_block.is_empty() {
        preamble
    } else {
        format!(
            "{preamble}\n\nRelevant excerpts from earlier in this conversation:\n{recall_block}"
        )
    };

    let client = openai::Client::from_env().map_err(|e| ServerFnError::new(e.to_string()))?;
    let agent = client
        .agent("gpt-4o")
        .preamble(&effective_preamble)
        .build();

    let mut history: Vec<Message> = recent
        .iter()
        .map(|(role, content)| {
            if role == "user" {
                Message::user(content.clone())
            } else {
                Message::assistant(content.clone())
            }
        })
        .collect();

    let reply = agent
        .chat(&prompt, &mut history)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    memory::append_turns(
        &agent_id,
        &[
            ("user".to_string(), prompt),
            ("assistant".to_string(), reply.clone()),
        ],
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(reply)
}
