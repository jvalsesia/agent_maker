# syntax=docker/dockerfile:1.7

# ---------- Builder ----------
FROM rust:1.95-slim-bookworm AS builder

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates build-essential git curl \
    && rm -rf /var/lib/apt/lists/*

# wasm target for the client bundle
RUN rustup target add wasm32-unknown-unknown

# Pin dioxus-cli to a 0.7.x release matching the project's dioxus version.
ARG DIOXUS_CLI_VERSION=0.7.9
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
# Expected layout under the resolved dir:
#   ./server      (server binary)
#   ./public/     (static assets served by the binary)
RUN set -eux; \
    OUT="$(find target/dx -type d -name web | head -n1)"; \
    test -n "$OUT"; \
    echo "bundle output: $OUT"; ls -la "$OUT"; \
    test -x "$OUT/server" || { echo "ERROR: expected server binary at $OUT/server"; ls -la "$OUT"; exit 1; }; \
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

# dx bundle (web + server) writes the server binary named literally `server`
# at the root of the bundle, alongside a `public/` directory of static assets.
# The server expects to be run from the bundle root so it can find `public/`.
WORKDIR /app
CMD ["/app/server"]
