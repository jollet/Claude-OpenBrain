FROM rust:1.84-slim-bookworm AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy full workspace
COPY Cargo.toml ./
COPY brain-core/ brain-core/
COPY brain-cli/ brain-cli/

RUN cargo build --release -p brain-core

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/brain-core /usr/local/bin/
RUN mkdir -p /data
CMD ["brain-core"]
