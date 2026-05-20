CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE IF NOT EXISTS chat_turns (
    id        UUID        PRIMARY KEY,
    agent_id  TEXT        NOT NULL,
    role      TEXT        NOT NULL,
    content   TEXT        NOT NULL,
    ts_ms     BIGINT      NOT NULL,
    embedding vector(1536) NOT NULL
);

CREATE INDEX IF NOT EXISTS chat_turns_agent_ts_idx
    ON chat_turns(agent_id, ts_ms);

CREATE INDEX IF NOT EXISTS chat_turns_embedding_idx
    ON chat_turns USING hnsw (embedding vector_cosine_ops);
