FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

ARG TARGETPLATFORM
COPY release-artifacts/linux-amd64/pixoo-bridge /usr/local/bin/pixoo-bridge-amd64
COPY release-artifacts/linux-arm64/pixoo-bridge /usr/local/bin/pixoo-bridge-arm64
RUN if [ "$TARGETPLATFORM" = "linux/amd64" ]; then \
      mv /usr/local/bin/pixoo-bridge-amd64 /usr/local/bin/pixoo-bridge && \
      rm /usr/local/bin/pixoo-bridge-arm64; \
    else \
      mv /usr/local/bin/pixoo-bridge-arm64 /usr/local/bin/pixoo-bridge && \
      rm /usr/local/bin/pixoo-bridge-amd64; \
    fi

## The listener honors PIXOO_BRIDGE_PORT (default 4000)
EXPOSE 4000

HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
  CMD curl -fsS http://localhost:4000/health || exit 1

USER 1000:1000

CMD ["/usr/local/bin/pixoo-bridge"]
