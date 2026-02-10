FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

ARG TARGETARCH
COPY release-artifacts/linux-${TARGETARCH}/pixoo-bridge /usr/local/bin/pixoo-bridge
RUN chmod +x /usr/local/bin/pixoo-bridge

## The listener honors PIXOO_BRIDGE_PORT (default 4000)
EXPOSE 4000

HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
  CMD curl -fsS http://localhost:${PIXOO_BRIDGE_PORT:-4000}/health || exit 1

USER 1000:1000

CMD ["/usr/local/bin/pixoo-bridge"]
