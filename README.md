# agent_maker

A Dioxus 0.7 fullstack app for building and chatting with simple LLM agents.

The home page shows a dashboard of agent cards loaded from Postgres. Click **+ New Agent** to create one (name + preamble + initial prompt are persisted server-side), or click any card to open a full-screen chat window that streams replies through an OpenAI model via [`rig-core`](https://crates.io/crates/rig-core) on the server side. The client never sees the API key — calls go through Dioxus server functions.

## Project layout

```
agent_maker/
├─ assets/                 # favicon, tailwind.css, main.css
├─ src/
│  ├─ main.rs              # entry point, Route enum, App component
│  ├─ components/
│  │  ├─ mod.rs
│  │  ├─ home.rs           # Home → renders Dashboard
│  │  ├─ dashboard.rs      # Grid of AgentCards; loads list via list_agents, hosts NewAgentModal
│  │  ├─ agent_card.rs     # Single agent card (avatar, name, preamble, Open/Edit)
│  │  ├─ new_agent_modal.rs # Modal form for creating a new agent
│  │  ├─ ui.rs             # Shared primitives: Button, Card, Heading, Label
│  │  ├─ chat_window.rs    # Full-screen wrapper around ChatComponent
│  │  ├─ chat.rs           # Chat UI; calls chat_with_llm server fn
│  │  ├─ navbar.rs         # Shared navbar (route layout)
│  │  └─ blog.rs           # Demo /blog/:id route
│  ├─ models/
│  │  └─ agent_model.rs    # AgentModel { id, name, preamble, prompt, response }
│  ├─ memory.rs            # Postgres + pgvector: chat_turns + agents tables
│  └─ server_fns.rs        # #[server] list_agents, create_agent, load_history, chat_with_llm
├─ Cargo.toml
├─ Dioxus.toml
├─ clippy.toml             # bans GenerationalRef / WriteLock across await
├─ AGENTS.md               # Dioxus 0.7 API cheat sheet
└─ CLAUDE.md               # Guidance for Claude Code
```

## Features

- `web` (default) — client-side wasm bundle.
- `server` (default) — Axum server binary, pulls in `rig-core` for OpenAI calls and `sqlx` + `pgvector` for chat memory.
- `desktop` / `mobile` — alternate platform builds (no `rig-core`, no memory layer).

Server-only dependencies (`rig-core`, `sqlx`, `pgvector`, `tokio`, `anyhow`) are `optional` and only enabled by the `server` feature, so they never land in the wasm output.

## Chat memory (Postgres + pgvector)

Per-agent chat history lives in Postgres. Each user/assistant turn is stored with an OpenAI `text-embedding-ada-002` embedding (1536 dims) in a single `chat_turns` table indexed by `agent_id`. On each chat call, the most recent turns are sent verbatim and older turns are pulled in via top-k cosine similarity (`recall`) so the agent has long-term context without blowing up the prompt.

Agents themselves are also persisted, in an `agents` table (`id TEXT PRIMARY KEY, name, preamble, prompt, created_ms`). The id is a server-side UUID and matches `chat_turns.agent_id`. Migration `0002_agents.sql` seeds a `General Assistant` row so the dashboard is never empty on a fresh DB.

**Requirements**

- A reachable Postgres database (local, Docker, or managed).
- The [`pgvector`](https://github.com/pgvector/pgvector) extension installed on that database (pgvector ≥ 0.5.0 for the HNSW index used by the bootstrap; downgrade the index to `ivfflat` if you're on an older version).
- `DATABASE_URL` exported in the server environment.

**Quick start with Docker**

```bash
docker run -d --name agent-pg \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=agent_maker \
  -p 5432:5432 \
  pgvector/pgvector:pg16

export DATABASE_URL=postgres://postgres:postgres@localhost:5432/agent_maker
```

The schema lives in [`migrations/`](migrations/) and is applied automatically on first DB connect via `sqlx::migrate!()` — no manual step needed. The initial migration enables the `vector` extension and creates the `chat_turns` table plus an HNSW cosine index. If your DB role can't `CREATE EXTENSION`, install pgvector once as a superuser before starting the server (the `pgvector/pgvector` Docker image already includes it).

## Docker

A multi-stage [`Dockerfile`](Dockerfile) builds the production web bundle with `dx bundle --platform web --release`, and [`docker-compose.yml`](docker-compose.yml) wires up Postgres (with pgvector) alongside the app.

```bash
cp .env.example .env       # then edit OPENAI_API_KEY (and credentials if you like)
docker compose up --build
```

The app listens on `http://localhost:${APP_PORT:-8080}`. Postgres data persists in the `pgdata` volume. Migrations run automatically on app startup.

To rebuild from clean state:

```bash
docker compose down -v     # also drops the pgdata volume
docker compose up --build
```

## Running

```bash
export OPENAI_API_KEY=sk-...
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/agent_maker
dx serve                       # web (default)
dx serve --platform desktop    # or mobile
```

Without `OPENAI_API_KEY` the dashboard still renders but `chat_with_llm` will error. Without `DATABASE_URL` (or if Postgres is unreachable) both `load_history` and `chat_with_llm` will return an error — the UI keeps working, just without persistence.

### Troubleshooting

**`Error: error running server function: DATABASE_URL must be set for chat memory`**

The server process couldn't read `DATABASE_URL` from its environment. `dx serve` does **not** auto-load `.env` files — it only inherits env vars from the shell that launched it. Fix it by exporting both vars in the same shell, then restarting `dx serve`:

```bash
export OPENAI_API_KEY=sk-...
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/agent_maker
dx serve
```

Verify the shell actually has them before launching:

```bash
echo "$DATABASE_URL"
# postgres://postgres:postgres@localhost:5432/agent_maker
```

Or set them inline for a one-shot run:

```bash
OPENAI_API_KEY=sk-... \
DATABASE_URL=postgres://postgres:postgres@localhost:5432/agent_maker \
dx serve
```

Note: the `.env` file is consumed by `docker compose` only. Don't copy the compose-style URL (`@postgres:5432/...`) into your host shell — `postgres` resolves only inside the compose network. From the host use `@localhost:5432/...`.

**Dashboard is empty / `Failed to load agents: relation "agents" does not exist`**

You're running against a database that was initialised before `0002_agents.sql` existed. `sqlx::migrate!()` will apply the new migration on next start, so usually it's enough to restart the server. If you're on Docker and the volume is stuck in a weird state, nuke it:

```bash
docker compose down -v   # drops the pgdata volume
docker compose up --build
```

Or apply it by hand against the existing DB:

```bash
psql "$DATABASE_URL" -f migrations/0002_agents.sql
```

**`relation "chat_turns" does not exist` / `type "vector" does not exist`**

The pgvector extension isn't installed on the target database. Either use the `pgvector/pgvector:pg16` image (recommended) or install pgvector manually and let the migrations run on next start. Verify with:

```bash
psql "$DATABASE_URL" -c "SELECT extname FROM pg_extension WHERE extname='vector';"
# expect one row: vector
```

**Connection refused / timeout**

Confirm Postgres is up and the port is reachable:

```bash
docker ps --filter name=agent-pg
psql "$DATABASE_URL" -c "SELECT 1;"
```

**`exec: "/app/server/agent_maker": stat /app/server/agent_maker: not a directory`**

You're on a stale image where the `Dockerfile` assumed `dx bundle` wrote the server binary under `server/<crate-name>`. Dioxus 0.7.x actually writes a binary literally named `server` at the bundle root (with assets in `public/`). The current `Dockerfile` uses `CMD ["/app/server"]` and asserts the binary exists during the build. Rebuild without cache to pick it up:

```bash
docker compose build --no-cache app
docker compose up -d
```

If the build itself fails with `ERROR: expected server binary at .../server`, your `dioxus-cli` version laid the bundle out differently — inspect with:

```bash
docker compose build --progress=plain app 2>&1 | grep -A3 "bundle output:"
```

and adjust `CMD` in the `Dockerfile` to match the printed listing.

### Tailwind

As of Dioxus 0.7, `dx serve` compiles Tailwind automatically by picking up `tailwind.css` (next to `Cargo.toml`, or under `assets/`). No `npx tailwindcss --watch` step is required.

To customise the input/output paths, edit `Dioxus.toml`:

```toml
[application]
tailwind_input = "my.css"
tailwind_output = "assets/out.css"
```

## Development commands

```bash
cargo build                      # default features: web + server
cargo build --features desktop
cargo clippy --all-targets
cargo fmt
```

See [CLAUDE.md](CLAUDE.md) for architecture notes and [AGENTS.md](AGENTS.md) for the Dioxus 0.7 API reference this project is written against.
