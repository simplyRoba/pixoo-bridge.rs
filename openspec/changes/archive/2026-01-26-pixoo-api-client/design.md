## Context

The Pixoo device expects JSON-like POST bodies with a `Command` field plus a variable set of fields that depend on the command. Responses always include `error_code`, may include additional fields, and often arrive with incorrect `Content-Type` headers, so callers must treat payloads as JSON regardless of headers. The bridge is a single-crate Rust service with minimal dependencies, and Pixoo endpoints can be flaky, so HTTP handling should stay thin with retry/backoff isolated in a dedicated layer.

## Goals / Non-Goals

**Goals:**
- Provide a reusable Pixoo HTTP client that accepts a command enum plus arbitrary arguments and returns response fields after validating `error_code`.
- Parse responses as JSON even when the server returns incorrect content types.
- Keep dependencies minimal and the HTTP layer explicit, with retry/backoff and error handling centralized.

**Non-Goals:**
- Implement the full catalog of Pixoo commands or strongly typed per-command request/response structures in this change.
- Add long-lived connection pooling, batching, or advanced resilience strategies beyond simple retries.

## Decisions

- Represent commands as an enum for the `Command` field, with extra fields provided as `serde_json::Map<String, Value>` that gets flattened into the request payload. This keeps the command type explicit while allowing variable arguments per command.
  - Alternative: define a struct per command and a large enum of variants with typed payloads. Rejected for now because it adds heavy surface area before requirements stabilize.
- Parse responses by reading the body as text and deserializing JSON manually, ignoring the HTTP `Content-Type` header. This matches the device's inconsistent behavior and avoids brittle header checks.
  - Alternative: rely on standard JSON response parsing based on headers. Rejected because Pixoo returns `text/html` in practice.
- Introduce a small Pixoo client module that encapsulates request building, HTTP execution, and response validation, with a retry/backoff helper for transient network errors and non-200 responses.
  - Alternative: inline HTTP calls at each call site. Rejected to avoid duplication and inconsistent error handling.
- Add minimal dependencies for JSON serialization (`serde`, `serde_json`) and a lightweight HTTP client (`reqwest` or hyper-based client) consistent with existing runtime dependencies.
  - Alternative: manual JSON formatting and ad-hoc HTTP handling. Rejected for correctness and maintainability.

## Risks / Trade-offs

- Parsing arbitrary JSON into a generic map trades strict typing for flexibility. → Mitigation: keep command enum explicit, validate required fields for critical commands in higher-level wrappers.
- Simple retry/backoff could still amplify load or delay error surfacing. → Mitigation: cap retries, use small backoff intervals, and only retry on network/transport failures.
- Adding `serde`/HTTP client increases binary size slightly. → Mitigation: stick to minimal feature sets and avoid extra crates.

## Migration Plan

1. Add the Pixoo client module with request/response types and error handling.
2. Update existing call sites to use the new client for sending commands and parsing responses.
3. Keep behavior compatible by preserving request shape and response validation, then iterate on typed helpers as needed.
4. Rollback by reverting call sites to the previous direct HTTP logic if issues surface.

## Open Questions

- Should we standardize a base endpoint path (e.g., `/post`) or make it configurable per device?
- Do we want a small set of typed wrappers for frequently used commands now, or defer until command usage stabilizes?
