//! Per-agent chat memory backed by Postgres + pgvector.
//!
//! Stores every chat turn with an OpenAI embedding of its content. Provides
//! `append_turns` (persist), `load_history` (verbatim transcript ordered by
//! timestamp) and `recall` (top-k semantic search over older turns).
//!
//! Connection comes from `DATABASE_URL`. The `vector` extension must be
//! installed on the target database (`CREATE EXTENSION vector;`). Schema is
//! bootstrapped on first connect; all turns share a single `chat_turns` table
//! keyed by `agent_id`.

use anyhow::{Context, Result};
use pgvector::Vector;
use rig_core::client::{EmbeddingsClient, ProviderClient};
use rig_core::embeddings::EmbeddingModel;
use rig_core::providers::openai;
use sqlx::postgres::{PgPool, PgPoolOptions};
use tokio::sync::OnceCell;
use uuid::Uuid;

const EMBED_MODEL: &str = openai::TEXT_EMBEDDING_ADA_002;
// Embedding dimensionality is enforced by the `vector(1536)` column in
// migrations/0001_init.sql — keep them in sync if you change EMBED_MODEL.
#[allow(dead_code)]
const EMBED_DIMS: usize = 1536;

static POOL: OnceCell<PgPool> = OnceCell::const_new();

async fn pool() -> Result<&'static PgPool> {
    POOL.get_or_try_init(|| async {
        let url =
            std::env::var("DATABASE_URL").context("DATABASE_URL must be set for chat memory")?;
        let pool = PgPoolOptions::new()
            .max_connections(8)
            .connect(&url)
            .await
            .context("failed to connect to Postgres")?;
        bootstrap(&pool).await?;
        Ok::<_, anyhow::Error>(pool)
    })
    .await
}

async fn bootstrap(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .context("running chat memory migrations")?;
    Ok(())
}

/// Persist `turns` (role, content) for `agent_id`, embedding each content.
pub async fn append_turns(agent_id: &str, turns: &[(String, String)]) -> Result<()> {
    if turns.is_empty() {
        return Ok(());
    }

    let client = openai::Client::from_env()?;
    let model = client.embedding_model(EMBED_MODEL);

    let texts: Vec<String> = turns.iter().map(|(_, c)| c.clone()).collect();
    let embeddings = model.embed_texts(texts).await?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis() as i64;

    let pool = pool().await?;
    let mut tx = pool.begin().await?;

    for (i, ((role, content), emb)) in turns.iter().zip(embeddings.iter()).enumerate() {
        let vec_f32: Vec<f32> = emb.vec.iter().map(|x| *x as f32).collect();
        let embedding = Vector::from(vec_f32);
        sqlx::query(
            "INSERT INTO chat_turns (id, agent_id, role, content, ts_ms, embedding)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(Uuid::new_v4())
        .bind(agent_id)
        .bind(role)
        .bind(content)
        .bind(now + i as i64)
        .bind(embedding)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

/// Full transcript for `agent_id`, ordered by insertion time (oldest first).
pub async fn load_history(agent_id: &str) -> Result<Vec<(String, String)>> {
    let pool = pool().await?;
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT role, content FROM chat_turns
         WHERE agent_id = $1
         ORDER BY ts_ms ASC",
    )
    .bind(agent_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// A row from the `agents` table.
///
/// Mirrors the persisted shape: client-side [`AgentModel`](crate::models::agent_model::AgentModel)
/// adds a transient `response` field that lives only in the UI.
#[derive(Debug, Clone)]
pub struct AgentRow {
    pub id: String,
    pub name: String,
    pub preamble: String,
    pub prompt: String,
}

/// Return every persisted agent, ordered oldest-first by `created_ms`
/// (ties broken by `id`).
pub async fn list_agents() -> Result<Vec<AgentRow>> {
    let pool = pool().await?;
    let rows: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT id, name, preamble, prompt FROM agents
         ORDER BY created_ms ASC, id ASC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, name, preamble, prompt)| AgentRow {
            id,
            name,
            preamble,
            prompt,
        })
        .collect())
}

/// Insert a new agent. The `id` is a server-side [`Uuid::new_v4`] so the
/// caller cannot poison it, and `created_ms` is the current Unix time in
/// milliseconds. Returns the inserted row so the caller doesn't need a
/// follow-up `SELECT`.
pub async fn create_agent(name: &str, preamble: &str, prompt: &str) -> Result<AgentRow> {
    let id = Uuid::new_v4().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis() as i64;
    let pool = pool().await?;
    sqlx::query(
        "INSERT INTO agents (id, name, preamble, prompt, created_ms)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&id)
    .bind(name)
    .bind(preamble)
    .bind(prompt)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(AgentRow {
        id,
        name: name.to_string(),
        preamble: preamble.to_string(),
        prompt: prompt.to_string(),
    })
}

/// Top-k semantically-similar past turns for `query` within `agent_id`.
pub async fn recall(agent_id: &str, query: &str, k: usize) -> Result<Vec<(String, String)>> {
    let client = openai::Client::from_env()?;
    let model = client.embedding_model(EMBED_MODEL);
    let emb = model.embed_text(query).await?;
    let qvec: Vec<f32> = emb.vec.iter().map(|x| *x as f32).collect();
    let qvec = Vector::from(qvec);

    let pool = pool().await?;
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT role, content FROM chat_turns
         WHERE agent_id = $1
         ORDER BY embedding <=> $2
         LIMIT $3",
    )
    .bind(agent_id)
    .bind(qvec)
    .bind(k as i64)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
