-- Reusable skills: plain-text instruction bundles that can be attached to
-- agents and concatenated into their preamble at chat time.
-- id is a UUID string (server-assigned), stored as TEXT for consistency with
-- agents.id and chat_turns.agent_id.
CREATE TABLE IF NOT EXISTS skills (
    id           TEXT   PRIMARY KEY,
    name         TEXT   NOT NULL,
    description  TEXT   NOT NULL DEFAULT '',
    instructions TEXT   NOT NULL,
    created_ms   BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS skills_created_idx
    ON skills(created_ms);

-- Many-to-many: an agent can have multiple skills; a skill can be reused
-- across agents. ON DELETE CASCADE keeps the join table tidy when either side
-- is removed.
CREATE TABLE IF NOT EXISTS agent_skills (
    agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    skill_id TEXT NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    PRIMARY KEY (agent_id, skill_id)
);

CREATE INDEX IF NOT EXISTS agent_skills_skill_idx
    ON agent_skills(skill_id);
