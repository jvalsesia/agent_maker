-- Persisted agents. One row per agent in the dashboard.
-- The id is a UUID string (generated server-side in memory::create_agent);
-- stored as TEXT to match the agent_id column on chat_turns and the
-- AgentModel.id field used by the UI/server functions.
-- created_ms is Unix epoch milliseconds, used solely for stable ordering.
CREATE TABLE IF NOT EXISTS agents (
    id        TEXT        PRIMARY KEY,
    name      TEXT        NOT NULL,
    preamble  TEXT        NOT NULL,
    prompt    TEXT        NOT NULL DEFAULT '',
    created_ms BIGINT     NOT NULL
);

CREATE INDEX IF NOT EXISTS agents_created_idx
    ON agents(created_ms);

-- Seed the default "General Assistant" so first-time users have something
-- to chat with. ON CONFLICT keeps the stable ID stable across redeploys.
INSERT INTO agents (id, name, preamble, prompt, created_ms)
VALUES (
    '00000000-0000-4000-8000-000000000001',
    'General Assistant',
    'You are a helpful assistant.',
    'Summarize the latest research on this topic.',
    0
)
ON CONFLICT (id) DO NOTHING;
