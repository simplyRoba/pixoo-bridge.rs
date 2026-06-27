## Context

The bridge currently emits three incompatible error base shapes:
- `{ "error": "validation failed", "details": {...} }` (400) and `{ "error": "not found" }` (404) — built by `json_error()`/`ErrorBuilder` in `src/routes/common.rs` and `src/routes/mod.rs`.
- `{ "error": "file too large", "limit", "actual" }` (413) — `payload_too_large()` in `src/routes/draw.rs`.
- `{ "error_status", "message", "error_kind", "error_code?" }` (500/502/503/504) — `PixooHttpErrorResponse` in `src/pixoo/error.rs`, already used after the OpenAPI change unified the 5xx paths.

A client cannot parse all errors with one shape. This change makes the root identical for every error and confines all variation to a nested `details` object.

## Goals / Non-Goals

**Goals:**
- One envelope for every error response: root is always `error_status`, `error_kind`, `message`.
- All kind-specific data nested under `details`, omitted when empty.
- A single OpenAPI schema referenced by every error response.

**Non-Goals:**
- No change to which HTTP status codes are returned, validation rules, or Pixoo behavior.
- No backward-compatible alias for the old `error` field (this is an accepted breaking change).

## Decisions

### Decision: Generalize `PixooHttpErrorResponse` into the one shared envelope
Extend the existing `PixooHttpErrorResponse` (already the 5xx body) rather than introduce a new type:
- Fields: `error_status: u16`, `error_kind: PixooHttpErrorKind`, `message: String`, and `details: Option<serde_json::Value>` with `#[serde(skip_serializing_if = "Option::is_none")]`.
- Extend `PixooHttpErrorKind` with `Validation`, `NotFound`, `PayloadTooLarge` (kebab-cased in serde to `validation`, `not-found`, `payload-too-large`).
- Provide constructors: `new(status, kind, message)` (no details) and `with_details(status, kind, message, details)`.

- **Why**: The 5xx envelope is already the most structured and is referenced across the OpenAPI docs. Reusing it means one schema, one `into_response()`, and no duplicate types.
- **Alternative considered**: A brand-new `ErrorEnvelope` type plus a `From` for the Pixoo one — rejected as redundant; it would duplicate the schema and constructors.

### Decision: `details` is a free-form `serde_json::Value` object, omitted when empty
Each producer supplies the case-specific object: validation → the field/action error map; payload-too-large → `{ "limit", "actual" }`; device error → `{ "error_code" }` when present.

- **Why**: Validation details are already a dynamic map; forcing a typed struct per case would fragment the schema again. The spec only mandates the root be uniform — `details` is intentionally open.
- **Trade-off**: `details` is `type: object` (loosely typed) in OpenAPI. Acceptable: the docs describe the per-kind contents in prose, and the root — the part clients branch on — is fully typed.

### Decision: Retire `json_error()`/`ErrorBuilder` and route all producers through the envelope
Rewrite `validation_error_simple`, `action_validation_error`, `validation_errors_response` (common.rs), `not_found` (mod.rs), and `payload_too_large`/`remote_fetch_failed` (draw.rs) to build `PixooHttpErrorResponse`. Remove `json_error`/`ErrorBuilder` once unused.

- **Why**: A single construction path prevents future drift back into ad-hoc shapes.

### Decision: Collapse OpenAPI error schemas to one
Drop `ValidationErrorBody` and `PayloadTooLargeBody` from `src/openapi.rs`; reference `PixooHttpErrorResponse` for all 400/404/413/500/502/503/504 responses. Keep the coherent struct-level `#[schema(example = ...)]`.

## Risks / Trade-offs

- [Breaking change for existing clients parsing `error`/`limit`/`actual`/`error_code` at root] → Documented in README + Kotlin migration notes; `error_kind` gives clients a clean discriminator going forward.
- [`details` loosely typed in the spec] → Root is strongly typed; per-kind `details` contents documented in prose. Acceptable trade-off for one stable envelope.
- [Many tests assert the old shape] → Updated as part of the change; the test suite is the safety net proving the new shape everywhere.

## Migration Plan

1. Extend `PixooHttpErrorKind` and add `details` + constructors to `PixooHttpErrorResponse`.
2. Rewrite every error producer to emit the envelope; remove `json_error`/`ErrorBuilder`.
3. Collapse OpenAPI schemas; update all `#[utoipa::path]` error `body = ...` references.
4. Update all tests to assert the new shape (`error_kind`, `message`, nested `details`).
5. Update README error docs + migration notes.
6. Run `cargo fmt`, `cargo clippy`, `cargo test`.

**Rollback**: revert the change; no persisted state involved.

## Open Questions

- None.
