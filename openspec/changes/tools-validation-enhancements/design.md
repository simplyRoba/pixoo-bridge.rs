## Context

The Pixoo bridge exposes the tools routes as HTTP handlers that currently parse raw strings and numbers inside each handler body. Every handler re-implements range checks and string matching, which makes the code noisy and hides the intent of the Pixoo command wiring. The rest of the crate already assumes handlers hand it validated commands, so the HTTP layer should share validation logic across future tool routes instead of repeating ad-hoc parsing for every new capability.

## Goals / Non-Goals

**Goals:**
- Define strongly typed request models that derive `Deserialize` and `Validate` so every tool extractor enforces range/action constraints before handler logic runs.
- Replace string-based action path parameters with enums that serde parses via lowercase variants so unsupported values fail during extraction across all tool actions.
- Keep existing JSON responses untouched while adding clear error handling for validation failures (HTTP 400 with context) and document how Pixoo retries can reuse the same normalized payloads.

**Non-Goals:**
- Reworking Pixoo transport, UDP framing, or other HTTP routes that already have dedicated validation guards.
- Introducing a new validation framework beyond tight attribute-based helpers; rely on the minimal `validator` dependency the crate already tolerates.

## Decisions

1. **Extractor-centric validation** → Define shared tool request structs that derive `Deserialize`, `Validate`, and `JsonSchema` (if already in use) so every extractor normalizes values through attribute-based ranges instead of embedding checks inside handlers.
2. **Enum actions with serde lowercase** → Introduce shared action enums that derive `Deserialize` with `#[serde(rename_all = "lowercase")]` so unsupported values fail during extraction regardless of which tool route is called, and reuse those enums across handlers/tests for consistency.
3. **Validation failure handling** → Wire `validator` errors into a shared HTTP error conversion returning a 400 response with a short `error` string and `details` map (field → message), keeping handler responses stable while surfacing malformed payloads.
4. **Minimal dependency impact** → Add `validator` (and optionally `serde_aux` or `derive_more` if needed) only with the features compatible with the Docker image and reuse attribute-based helpers to avoid heavyweight validation frameworks.

## Risks / Trade-offs

- [New dependency size] → `validator` adds a bit of compile time and image size; mitigation: enable only the features we need, keep the crate version aligned with the rest of the stack, and lock it in Cargo.lock to avoid unexpected transitive dependencies.
- [Error messaging drift] → Changing validation could cause tests expecting old manual error messages to fail; mitigation: keep the JSON response format (`{ "error": "...", "details": { ... } }`) unchanged while translating validator errors into that shape.
- [Enum coverage] → If a Pixoo action is added later, we risk panics if the enum is forgotten; mitigation: document in the spec that new actions require enum updates and add compile-time `#[non_exhaustive]` markers or explicit tests to catch missing variants.

## Migration Plan

1. Add the request models/enums, deriving validation and serde traits.
2. Update `src/routes/tools.rs` to take the new extractors and rely on their fields; keep existing JSON responses by translating extractor errors into the shared error type.
3. Expand the tools route tests to cover valid requests, invalid ranges, and invalid actions while keeping response payloads the same as before.
4. Run `cargo fmt`, `cargo clippy`, and `cargo test` to ensure validators and enums compile cleanly.

## Open Questions

- Should the shared validation error conversion also log which field failed so that flaky Pixoo payloads can be traced, or is surface-level JSON enough?
- Do we need to expose these validated models elsewhere (e.g., a CLI or future automation APIs), or keep them scoped to the HTTP handlers for now?
