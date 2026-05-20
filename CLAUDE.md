# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Dioxus 0.7 fullstack app (`agent_maker`) — a small "agent dashboard" that lets the user open per-agent chat windows backed by an OpenAI LLM via [`rig-core`](https://crates.io/crates/rig-core).

## Commands

- Run dev server (default = web): `dx serve`
- Other platforms: `dx serve --platform desktop` / `--platform mobile`
- Build: `cargo build` (add `--features desktop` / `--features mobile` as needed; default features are `web` + `server`)
- Lint: `cargo clippy --all-targets` (see [clippy.toml](clippy.toml) — bans holding `GenerationalRef` / `WriteLock` across `await`)
- Format: `cargo fmt`
- LLM env: the `chat_with_llm` server function reads `OPENAI_API_KEY` via `openai::Client::from_env()` — export it before `dx serve` when exercising chat.
- Memory env: `DATABASE_URL` (Postgres) is required by [src/memory.rs](src/memory.rs). The `vector` extension must be available on the target DB; the schema lives in [migrations/](migrations/) and is applied via `sqlx::migrate!()` on first connect.
- Docker: `cp .env.example .env && docker compose up --build` brings up Postgres (pgvector) + the app on `http://localhost:${APP_PORT:-8080}`. `docker compose down -v` wipes the `pgdata` volume.

Tailwind is handled automatically by `dx serve` (picks up [tailwind.css](assets/tailwind.css) — no separate `npx tailwindcss` needed in Dioxus 0.7+).

## Architecture

- **Entry point**: `dioxus::launch(App)` in [src/main.rs](src/main.rs). `App` mounts stylesheets via `document::Link` and renders `Router::<Route>`.
- **Routing**: single `Route` enum derives `Routable`. `#[layout(Navbar)]` wraps all routes; `Navbar` renders `Outlet::<Route> {}`. Routes: `Home {}` (the agents dashboard) and `Blog { id: i32 }`.
- **Module layout**:
  - [src/components/](src/components/) — UI components, re-exported through [src/components/mod.rs](src/components/mod.rs):
    - `Home` → wraps `Dashboard`.
    - `Dashboard` — grid of `AgentCard`s; clicking an agent swaps the view to a full-screen `ChatWindow`.
    - `AgentCard` — card with avatar, name, preamble preview, Open/Edit actions.
    - `ChatWindow` — header + responsive container that hosts `ChatComponent`.
    - `ChatComponent` — message list + input; calls `chat_with_llm` server function and preserves history as `Vec<ChatTurn>`.
    - `Navbar`, `Blog` — shared chrome and demo route.
  - [src/models/agent_model.rs](src/models/agent_model.rs) — `AgentModel { id, name, preamble, prompt, response }`, `Serialize + Deserialize + Clone + PartialEq`.
  - [src/server_fns.rs](src/server_fns.rs) — server-only LLM glue. `ChatTurn { role, content }`, `#[server] load_history(agent_id)` and `#[server] chat_with_llm(agent_id, preamble, prompt)` built on `rig_core::providers::openai` with model `gpt-4o`. Recent turns are sent verbatim (`RECENT_WINDOW = 12`); older turns are surfaced via top-k semantic recall (`RECALL_K = 4`) and injected into the preamble.
  - [src/memory.rs](src/memory.rs) — per-agent chat memory on Postgres + pgvector. Lazy `PgPool` via `tokio::sync::OnceCell`, bootstrapped schema (`chat_turns(id, agent_id, role, content, ts_ms, embedding vector(1536))` + HNSW cosine index). Public API: `append_turns`, `load_history`, `recall`. Embeddings come from OpenAI `text-embedding-ada-002` (cast to `f32` for pgvector); change `EMBED_MODEL` / `EMBED_DIMS` together if you switch models.
- **Fullstack split**: features `web` (default) and `server` (default) compile different binaries from the same source. `rig-core` is gated behind `server` so it never enters the wasm bundle. Server functions become HTTP calls on the client and endpoints on the server — keep them callable from both sides.
- **Assets**: reference via the `asset!("/assets/...")` macro (paths relative to the project root, always start with `/`). Files live in [assets/](assets/).
- **State**: Dioxus 0.7 idioms only — `use_signal`, `use_memo`, `use_resource`, `use_context_provider` / `use_context`. The old `cx`, `Scope`, and `use_state` APIs are gone. Use `use_server_future` (not `use_resource`) when the value must be available on the first hydrated render to avoid client/server divergence.
- **Props**: must be owned (`String`, `Vec<T>`), `Clone + PartialEq`. Wrap in `ReadOnlySignal<T>` to make them reactive without losing `Copy`. Event callbacks use `EventHandler<T>` (see `AgentCard::on_open`, `ChatWindow::on_close`).

## Deployment

- [Dockerfile](Dockerfile) — multi-stage build.
  - **Builder**: `rust:1.83-slim-bookworm`, installs `dioxus-cli` (pinned via `DIOXUS_CLI_VERSION` ARG, default `0.7.0`), adds the `wasm32-unknown-unknown` target, and runs `dx bundle --platform web --release`. A dummy `src/main.rs` is compiled first to cache the dependency graph before real sources are copied.
  - **Runtime**: `debian:bookworm-slim` with `ca-certificates` + `libssl3` only. Runs as non-root user `app` (uid 10001), listens on `0.0.0.0:8080`. `CMD` is `/app/server/agent_maker` — adjust if `dx bundle` lays the binary out differently in your `dioxus-cli` version.
  - Migrations are both **embedded in the binary** (via `sqlx::migrate!`) and **copied into `/app/migrations`** so they can be inspected or replayed manually inside the container.
- [docker-compose.yml](docker-compose.yml) — two services:
  - `postgres`: image `pgvector/pgvector:pg16`, persisted to the `pgdata` named volume, healthchecked with `pg_isready`. Credentials and DB name come from `.env`.
  - `app`: built from the local `Dockerfile`, `depends_on: postgres (service_healthy)`, gets `DATABASE_URL` pointing at the compose-internal `postgres` hostname and `OPENAI_API_KEY` from `.env` (required — compose fails fast if unset).
- [.env.example](.env.example) — canonical env shape: `OPENAI_API_KEY`, `POSTGRES_USER`/`PASSWORD`/`DB`, `APP_PORT`, `RUST_LOG`. Copy to `.env` before `docker compose up`.
- [.dockerignore](.dockerignore) — keeps `target/`, `data/`, `.git`, docs, and env files out of the build context to keep image builds reproducible and fast.
- [migrations/](migrations/) — sqlx-compatible SQL migrations. `0001_init.sql` enables `vector`, creates `chat_turns`, and adds the HNSW cosine index. Add new files as `000N_<name>.sql`; they're applied in lexicographic order by `sqlx::migrate!()` on next server start.

## Operational notes

- Schema changes: add a new file under `migrations/`, never edit applied ones. `sqlx::migrate!` records hashes in `_sqlx_migrations` and will refuse to start if a prior migration's content changed.
- Changing the embedding model: update `EMBED_MODEL` in [src/memory.rs](src/memory.rs) **and** add a migration altering the `embedding` column dimension (pgvector requires a fixed-size column; existing rows must be re-embedded or dropped).
- Local dev without Docker: run `pgvector/pgvector:pg16` directly (`docker run -d -p 5432:5432 ...`) and export `DATABASE_URL` before `dx serve`. The lazy `PgPool` means the server starts even if Postgres is down — the failure surfaces only on first chat call.
- **`dx serve` does NOT auto-load `.env`.** Env vars must be exported in the shell that launches it, or passed inline (`DATABASE_URL=... OPENAI_API_KEY=... dx serve`). The error `DATABASE_URL must be set for chat memory` from [src/memory.rs](src/memory.rs) is always this — the `.env` file is consumed only by `docker compose`. Don't copy the compose URL (`@postgres:5432/...`) into a host shell; the `postgres` hostname only resolves inside the compose network — from the host use `@localhost:5432/...`.

See [AGENTS.md](AGENTS.md) for the full Dioxus 0.7 API cheat sheet this project is written against.
