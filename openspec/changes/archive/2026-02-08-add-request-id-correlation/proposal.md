## Why

Correlating production logs with user requests is currently impossible because outbound Pixoo interactions and handler logs lack a shared identifier. That makes troubleshooting intermittent failures and tracking a request’s lifecycle slow and error prone.

## What Changes

- Add middleware that generates a per-request identifier (e.g., `X-Request-Id`), stores it in the request extensions, injects it into responses, and propagates it through `tracing` spans so every log entry carries the same id.
- Instrument route handlers and Pixoo client interactions so they read the request id from extensions, annotate spans (`#[tracing::instrument]`), enrich log events, and include the identifier when sending payloads or logging errors.
- Ensure outgoing Pixoo requests and the HTTP response (header/body) re-use the identifier, allowing operators to trace a complete round trip from HTTP request to Pixoo device response and back.
- Document the new observability behavior in the logging capabilities so operators know what fields show up in the logs and headers.

## Capabilities

### New Capabilities

### Modified Capabilities
- `core/logging`: add requirements that every log entry and Pixoo interaction includes the request identifier so operators can correlate logs with HTTP requests and device responses.

## Impact

- `src/main.rs`, `src/routes/*`, and `src/state.rs` gain request-id-aware middleware and state extension plumbing.
- `src/pixoo/client.rs` and logging helpers will add the identifier to Pixoo command logs and errors.
- New tracing spans and middleware will touch configuration/observability docs and logging expectations in `openspec/specs/core/logging/specs.md`.
