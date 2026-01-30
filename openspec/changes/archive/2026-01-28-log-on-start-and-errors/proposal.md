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
- `startup-logging` (core): Capture startup configuration (listener address, health forwarding flag, Pixoo host) in a deterministic info-level log and honor `PIXOO_BRIDGE_LOG_LEVEL` so operators can understand runtime settings without digging through the container.
- `error-logging` (core): Log unexpected Pixoo interactions at error level with `error_code`, HTTP status, retriable flags, and payload metadata so deployment logs surface failures even when the Pixoo device misbehaves.

### Modified Capabilities
- None.

## Impact

- Core HTTP bridge startup path (`src/main.rs` and any logging helpers) must emit these statements.
- Error-handling helpers need to attach contextual metadata when logging so the logs describe why the failure occurred.
