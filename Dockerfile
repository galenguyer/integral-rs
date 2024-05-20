FROM docker.io/rust:1.78-bookworm AS builder
WORKDIR /src/
RUN cargo init --bin
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release
COPY . /src/
RUN touch src/main.rs && cargo build --release

FROM docker.io/debian:bookworm-slim
RUN DEBIAN_FRONTEND=noninteractive apt-get update -y \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        tini \
        ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app/

COPY --from=builder /src/target/release/integral /app/
ENTRYPOINT ["/usr/bin/tini", "-v", "--", "/app/integral"]
