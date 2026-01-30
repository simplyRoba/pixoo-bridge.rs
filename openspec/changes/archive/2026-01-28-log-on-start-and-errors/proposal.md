## Why

The bridge container currently starts silently, making it hard to verify that configuration values (device host, timeout, etc.) are what the service is using, and unexpected situations disappear without useful context. Capturing startup details and logging unexpected errors with their metadata improves observability for operators running the Docker image.

## What Changes

- Log the resolved configuration (device endpoint, timeouts, and feature toggles) when the service starts so operators can confirm the runtime settings they rely on.
- Emit structured logs for unexpected conditions (invalid responses, HTTP errors, serialization failures) so those failures appear in container logs instead of being swallowed by the HTTP layer.
- Keep logging lightweight and tied to the existing Rust core runtime so the Docker image and downstream automation stacks can capture everything written to stdout/stderr.
- Introduce `PIXOO_BRIDGE_LOG_LEVEL` (default `info`) so operators can lower or raise the emitted verbosity without rebuilding the container.
- Document the available log-level values in the README so operators know how to adjust verbosity (debug/info/warn/error).

## Capabilities

### New Capabilities
- `logging` (core): Surface deterministic startup configuration logging, honor `PIXOO_BRIDGE_LOG_LEVEL`, and ensure unexpected Pixoo errors emit contextual logs so container output captures both the runtime settings and failure metadata operators need.

### Modified Capabilities
- None.

## Impact

- Core HTTP bridge startup path (`src/main.rs` and any logging helpers) must emit these statements.
- Error-handling helpers need to attach contextual metadata when logging so the logs describe why the failure occurred.
