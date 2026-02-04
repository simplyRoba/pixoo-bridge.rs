## Context

The bridge currently treats every Pixoo error path as an HTTP 503, which masks the distinction between an unreachable device, a healthy device reporting a failure, and timed-out requests. Automations and monitors therefore have to rely on JSON payloads or a fixed retry pattern, even though HTTP already provides semantics for gateway errors (502), service unavailability (503), and timeouts (504). With this change we keep the single Rust crate architecture, ship the same minimal Docker image, and avoid pulling in any additional binaries while still letting operators observe different failure modes via HTTP.

## Goals / Non-Goals

**Goals:**
- Introduce a reusable `core/http-error-mapping` capability that classifies Pixoo client outcomes into HTTP status codes and human-readable payloads.
- Ensure each exposed endpoint (`/health`, `/manage/*`, `/reboot`, etc.) consults the mapping before writing its response so callers see the more precise status even when downstream Pixoo behavior varies.
- Keep the HTTP layer thin, reusing the existing retry/backoff helpers and minimal dependencies expected in the current Docker image.

**Non-Goals:**
- Re-implementing Pixoo’s retry logic, reworking transport protocols, or building a separate proxy service.
- Changing status semantics for endpoints that are already intentionally 200/204 on success (only the error paths change).

## Decisions

- _Introduce HTTP error classification in the Pixoo error module (`pixoo::error`)._ Centralizing the mapping keeps the bridge consistent across handlers; duplicating the mapping per handler would risk divergence when we tweak retry limits or add new error kinds. Co-locating with `PixooError` avoids an extra module and keeps error handling cohesive.
- _Extend the Pixoo client error type to expose categories (unreachable, timeout, device error)._ The client already knows when it hit each failure; surfacing that classification lets the mapper translate outcomes to 502, 504, or 503 without re-running diagnostics.
- _Share a helper that emits structured payloads (status code, message, debug hints)._ This keeps the HTTP layer simple—each handler can call `map_pixoo_error` and return its result rather than embedding status logic inline.

## Risks / Trade-offs

- [Risk] The error classification may miss nuanced Pixoo responses, so we could map healthy-but-error responses to 504 unintentionally. → Mitigation: start with conservative defaults (fall back to 503 when unsure) and add logging so we can revisit mappings if operators flag odd behavior.
- [Risk] Exposing more precise status codes might break consumers expecting every failure to be 503. → Mitigation: document the change in release notes and keep 503 as the default catch-all so only confident cases return 502/504.

## Migration Plan

- Update the Pixoo client error enum and helpers so every endpoint can observe whether a request timed out, never connected, or the device responded with an error.
- Implement `map_pixoo_error` in `pixoo::error` that maps classifications to HTTP responses and structured bodies; consume it from `/health`, `/manage/*`, `/reboot`, and any other Pixoo-exposed handlers.
- Run `cargo fmt`, `cargo clippy`, and `cargo test`, then publish release notes that mention the new status semantics and the capability `core/http-error-mapping`.

## Open Questions

- Should endpoints expose the underlying Pixoo `error_code` inside the payload for troubleshooting, or only a generic message? (Default to error_code if available.)
