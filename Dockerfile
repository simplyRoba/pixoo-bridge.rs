FROM rust:1.92-slim as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

ARG TARGETARCH
RUN if [ "$TARGETARCH" = "amd64" ]; then \
      cargo build --release --target x86_64-unknown-linux-gnu; \
    elif [ "$TARGETARCH" = "arm64" ]; then \
      apt-get update && apt-get install -y gcc-aarch64-linux-gnu && \
      CC=aarch64-linux-gnu-gcc RUSTFLAGS='-Clinker=aarch64-linux-gnu-gcc' cargo build --release --target aarch64-unknown-linux-gnu; \
    fi

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

ARG TARGETARCH
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/pixoo-bridge /usr/local/bin/pixoo-bridge-amd64
COPY --from=builder /app/target/aarch64-unknown-linux-gnu/release/pixoo-bridge /usr/local/bin/pixoo-bridge-arm64
RUN if [ "$TARGETARCH" = "amd64" ]; then \
      mv /usr/local/bin/pixoo-bridge-amd64 /usr/local/bin/pixoo-bridge && \
      rm /usr/local/bin/pixoo-bridge-arm64; \
    else \
      mv /usr/local/bin/pixoo-bridge-arm64 /usr/local/bin/pixoo-bridge && \
      rm /usr/local/bin/pixoo-bridge-amd64; \
    fi

EXPOSE 8080

USER 1000:1000

CMD ["pixoo-bridge"]