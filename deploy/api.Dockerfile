# syntax=docker/dockerfile:1
# Builds the Rust services; pick one with --target api | fsp-mock | ats-mock | embedder.
# Context: repo root (docker compose -f deploy/compose.yml build).

# Builder and runtime must share a Debian release: a binary linked against a newer glibc
# (rust:1 is trixie-based) does not start on bookworm-slim.
FROM rust:1-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app
# sqlx macros check queries against the committed .sqlx/ metadata: no database at build time.
ENV SQLX_OFFLINE=true

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin api --bin fsp-mock --bin ats-mock --bin embedder

# The embedding model's weights at a pinned Hugging Face revision, each file checked against its
# sha256. EMBEDDER_FETCH=0 builds without network: the image then expects the weights mounted at
# EMBEDDER_MODEL_DIR (README, «Эмбеддер без сети»).
FROM debian:bookworm-slim AS model
ARG EMBEDDER_MODEL=multilingual-e5-small
ARG EMBEDDER_FETCH=1
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
COPY crates/embedder/fetch-model.sh /usr/local/bin/fetch-model.sh
RUN mkdir -p /models \
    && if [ "$EMBEDDER_FETCH" = "1" ]; then sh /usr/local/bin/fetch-model.sh "$EMBEDDER_MODEL" /models; fi

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system --gid 10001 app \
    && useradd --system --uid 10001 --gid app --no-create-home --shell /usr/sbin/nologin app
WORKDIR /app

FROM runtime AS fsp-mock
COPY --from=builder /app/target/release/fsp-mock /usr/local/bin/fsp-mock
USER 10001:10001
ENV FSP_MOCK_BIND=0.0.0.0:8090
EXPOSE 8090
ENTRYPOINT ["/usr/local/bin/fsp-mock"]

FROM runtime AS ats-mock
COPY --from=builder /app/target/release/ats-mock /usr/local/bin/ats-mock
USER 10001:10001
ENV ATS_MOCK_BIND=0.0.0.0:8082
EXPOSE 8082
ENTRYPOINT ["/usr/local/bin/ats-mock"]

FROM runtime AS api
COPY --from=builder /app/target/release/api /usr/local/bin/api
# A fresh named volume copies this directory's ownership, so the non-root API can write chunks.
RUN mkdir -p /var/lib/fsp/proctoring && chown -R 10001:10001 /var/lib/fsp && chmod 700 /var/lib/fsp/proctoring
USER 10001:10001
ENV API_BIND=0.0.0.0:8000
EXPOSE 8000
ENTRYPOINT ["/usr/local/bin/api"]

FROM runtime AS embedder
ARG EMBEDDER_MODEL=multilingual-e5-small
COPY --from=builder /app/target/release/embedder /usr/local/bin/embedder
COPY --from=model /models /models
USER 10001:10001
ENV EMBEDDER_BIND=0.0.0.0:8093 \
    EMBEDDER_MODEL=${EMBEDDER_MODEL} \
    EMBEDDER_MODEL_DIR=/models/${EMBEDDER_MODEL}
EXPOSE 8093
ENTRYPOINT ["/usr/local/bin/embedder"]
