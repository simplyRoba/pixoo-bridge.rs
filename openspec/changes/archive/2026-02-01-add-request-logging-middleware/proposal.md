## Why

Pixoo’s HTTP bridge currently lacks consistent request-level logging, making it difficult to understand when clients fail or why downstream Pixoo commands stall. Capturing every incoming request with a single layer would help debug client/api mismatches without scattering `tracing` calls across handlers.

## What Changes

- Add request-logging middleware around the entire router so every request and response emits method, path, status, and latency at DEBUG level without touching individual handlers.
- Document the new behavior briefly in the README so operators know request logs run at DEBUG and how to enable them via `PIXOO_BRIDGE_LOG_LEVEL`.
- Keep the existing handler behavior intact while adding instrumentation, so middleware only affects observability.

## Capabilities

### New Capabilities
- *none*

### Modified Capabilities
- `core/logging`: Add an access-log requirement so access entries include method, path, status, and duration for every request.

## Impact

- `src/main.rs`: wrap the router with `tower_http::trace::TraceLayer` (and any agreed-on service builder configuration) so per-request spans/log entries exist globally.
- `Cargo.toml`: add `tower-http` and `tower` if not already present to get access to `TraceLayer` and middleware helpers.
- `README.md`: mention that enriched request logging is available and where to see logs (if this change adds new guidance).
