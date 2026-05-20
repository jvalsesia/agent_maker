# syntax=docker/dockerfile:1.7

# ---------- Builder ----------
FROM rust:1.83-slim-bookworm AS builder

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates build-essential git curl \
    && rm -rf /var/lib/apt/lists/*

# wasm target for the client bundle
RUN rustup target add wasm32-unknown-unknown

# Pin dioxus-cli to a 0.7.x release matching the project's dioxus version.
ARG DIOXUS_CLI_VERSION=0.7.0
RUN cargo install dioxus-cli --version ${DIOXUS_CLI_VERSION} --locked

WORKDIR /app

# Cache deps first
COPY Cargo.toml Cargo.lock ./
COPY Dioxus.toml clippy.toml ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs \
 && cargo build --release --features server --no-default-features \
 ; rm -rf src

# Real sources + assets + migrations
COPY src ./src
COPY assets ./assets
COPY migrations ./migrations

# Production bundle (web client + server binary)
RUN dx bundle --platform web --release

# Locate the bundle output dir (dioxus-cli paths shift slightly across releases).
RUN set -eux; \
    OUT="$(find target/dx -type d -name web | head -n1)"; \
    test -n "$OUT"; \
    mkdir -p /out && cp -R "$OUT"/. /out/

# ---------- Runtime ----------
FROM debian:bookworm-slim AS runtime

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --system --create-home --uid 10001 app
WORKDIR /app

COPY --from=builder /out /app
# Migrations are embedded in the binary via sqlx::migrate!, but ship them too
# so they can be inspected / replayed manually from the container if needed.
COPY --from=builder /app/migrations /app/migrations

USER app
ENV PORT=8080 \
    IP=0.0.0.0 \
    RUST_LOG=info

EXPOSE 8080

# dx bundle places the server binary at the root of the web bundle directory.
# Name matches the [package] name in Cargo.toml.
CMD ["/app/server/agent_maker"]
