use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use crate::memory;
use crate::models::agent_model::AgentModel;
use crate::models::skill_model::SkillModel;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatTurn {
    pub role: String,
    pub content: String,
}

/// Server function: list all persisted agents (oldest first).
///
/// Maps each [`memory::AgentRow`] to a client-facing [`AgentModel`], leaving
/// the transient `response` field empty (it is populated by the UI as chat
/// progresses, never by the server).
#[server]
pub async fn list_agents() -> Result<Vec<AgentModel>, ServerFnError> {
    let rows = memory::list_agents()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows
        .into_iter()
        .map(|r| AgentModel {
            id: r.id,
            name: r.name,
            preamble: r.preamble,
            prompt: r.prompt,
            response: String::new(),
        })
        .collect())
}

/// Server function: persist a new agent.
///
/// The `id` and `created_ms` are assigned server-side, so any value the
/// client may have generated locally is discarded. Returns the canonical
/// [`AgentModel`] the caller should append to its in-memory list.
#[server]
pub async fn create_agent(
    name: String,
    preamble: String,
    prompt: String,
) -> Result<AgentModel, ServerFnError> {
    let row = memory::create_agent(&name, &preamble, &prompt)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(AgentModel {
        id: row.id,
        name: row.name,
        preamble: row.preamble,
        prompt: row.prompt,
        response: String::new(),
    })
}

/// Server function: list all persisted skills (oldest first).
#[server]
pub async fn list_skills() -> Result<Vec<SkillModel>, ServerFnError> {
    let rows = memory::list_skills()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows
        .into_iter()
        .map(|r| SkillModel {
            id: r.id,
            name: r.name,
            description: r.description,
            instructions: r.instructions,
            created_ms: r.created_ms,
        })
        .collect())
}

/// Server function: persist a new skill. `id` and `created_ms` are assigned
/// server-side.
#[server]
pub async fn create_skill(
    name: String,
    description: String,
    instructions: String,
) -> Result<SkillModel, ServerFnError> {
    let row = memory::create_skill(&name, &description, &instructions)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(SkillModel {
        id: row.id,
        name: row.name,
        description: row.description,
        instructions: row.instructions,
        created_ms: row.created_ms,
    })
}

/// Server function: update an existing skill in place.
#[server]
pub async fn update_skill(
    id: String,
    name: String,
    description: String,
    instructions: String,
) -> Result<SkillModel, ServerFnError> {
    let row = memory::update_skill(&id, &name, &description, &instructions)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(SkillModel {
        id: row.id,
        name: row.name,
        description: row.description,
        instructions: row.instructions,
        created_ms: row.created_ms,
    })
}

/// Server function: delete a skill. Cascades through `agent_skills`.
#[server]
pub async fn delete_skill(id: String) -> Result<(), ServerFnError> {
    memory::delete_skill(&id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

/// Attach a skill to an agent. Idempotent.
#[server]
pub async fn attach_skill(agent_id: String, skill_id: String) -> Result<(), ServerFnError> {
    memory::attach_skill(&agent_id, &skill_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

/// Detach a skill from an agent. Idempotent.
#[server]
pub async fn detach_skill(agent_id: String, skill_id: String) -> Result<(), ServerFnError> {
    memory::detach_skill(&agent_id, &skill_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

/// IDs of every skill attached to `agent_id`.
#[server]
pub async fn list_agent_skill_ids(agent_id: String) -> Result<Vec<String>, ServerFnError> {
    memory::list_agent_skill_ids(&agent_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
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

    let skills = memory::list_agent_skill_instructions(&agent_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let skill_block: String = skills
        .iter()
        .map(|(name, instr)| format!("## Skill: {name}\n{instr}"))
        .collect::<Vec<_>>()
        .join("\n\n");
    let preamble = if skill_block.is_empty() {
        preamble
    } else {
        format!("{preamble}\n\n{skill_block}")
    };

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
