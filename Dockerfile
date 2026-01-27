FROM rust:1.92-slim AS builder-amd64

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN apt-get update && apt-get install -y gcc-x86-64-linux-gnu && \
    rustup target add x86_64-unknown-linux-gnu && \
    CC_x86_64_unknown_linux_gnu=x86_64-linux-gnu-gcc \
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
    cargo build --release --target x86_64-unknown-linux-gnu

FROM rust:1.92-slim AS builder-arm64

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN apt-get update && apt-get install -y gcc-aarch64-linux-gnu && \
    rustup target add aarch64-unknown-linux-gnu && \
    CC=aarch64-linux-gnu-gcc RUSTFLAGS='-Clinker=aarch64-linux-gnu-gcc' \
    cargo build --release --target aarch64-unknown-linux-gnu

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

ARG TARGETARCH
COPY --from=builder-amd64 /app/target/x86_64-unknown-linux-gnu/release/pixoo-bridge /usr/local/bin/pixoo-bridge-amd64
COPY --from=builder-arm64 /app/target/aarch64-unknown-linux-gnu/release/pixoo-bridge /usr/local/bin/pixoo-bridge-arm64
RUN if [ "$TARGETARCH" = "amd64" ]; then \
      mv /usr/local/bin/pixoo-bridge-amd64 /usr/local/bin/pixoo-bridge && \
      rm /usr/local/bin/pixoo-bridge-arm64; \
    else \
      mv /usr/local/bin/pixoo-bridge-arm64 /usr/local/bin/pixoo-bridge && \
      rm /usr/local/bin/pixoo-bridge-amd64; \
    fi

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
  CMD curl -fsS http://localhost:8080/health || exit 1

USER 1000:1000

CMD ["pixoo-bridge"]
