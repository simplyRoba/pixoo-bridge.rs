## Why

Error responses currently come in three incompatible base shapes: validation/not-found/payload-too-large use `{ "error": ... }` (plus ad-hoc extras sprinkled at the root), while device/server failures use `{ "error_status", "message", "error_kind", "error_code?" }`. A client cannot write a single parser for errors — the discriminator field and even the base keys differ per status. This change makes every error response share one predictable root shape, with all case-specific data confined to a single nested object.

## What Changes

- Introduce a single canonical error envelope for **all** error responses (4xx and 5xx). The root object always has exactly these fields:
  - `error_status` (int) — the HTTP status, mirrored into the body
  - `error_kind` (string) — the discriminator
  - `message` (string) — human-readable text
- Place **all** kind-specific data in one optional nested object, `details`. No case-specific field ever appears at the root. `details` is **omitted entirely when empty**:
  - `details: { <field>: [<errors>] }` — validation failures (400)
  - `details: { "limit": <int>, "actual": <int> }` — payload too large (413)
  - `details: { "error_code": <int> }` — Pixoo device errors (503), when the device provides one
  - no `details` key — cases with no extra data (e.g. 404, timeouts, unreachable)
- Extend the `error_kind` discriminator to cover every error category: `validation`, `not-found`, `payload-too-large`, `unreachable`, `timeout`, `device-error`, `remote-fetch`, `internal`.
- **BREAKING**: Remove the legacy `error` string field, and move `limit`/`actual`/`error_code` off the root into `details`. Responses that previously returned `{ "error": "validation failed", "details": {...} }`, `{ "error": "not found" }`, and `{ "error": "file too large", "limit", "actual" }` now use the unified envelope (with `message` carrying the former `error` text, `error_kind` identifying the case, and extras nested under `details`).
- Update the OpenAPI documentation so all 4xx/5xx responses reference one error-envelope schema, replacing the separate `ValidationErrorBody` and `PayloadTooLargeBody` shapes.
- Update README's error documentation and the Kotlin migration notes.

Non-goals: no change to *which* HTTP status codes are returned, to validation rules, or to Pixoo command behavior. Only the error response body shape changes.

## Capabilities

### New Capabilities
<!-- None. This refines existing behavior. -->

### Modified Capabilities
- `api-common`: The "Structured HTTP error payloads for all mapped statuses" requirement broadens from device-mapped errors only to **every** error response (validation, not-found, payload-too-large, and device/server failures), all sharing the `error_status`/`error_kind`/`message` base with optional kind-specific fields.
- `api-docs`: The "Documented schemas cover request and response models" requirement changes so all error responses reference a single canonical error envelope schema instead of multiple divergent bodies.

## Impact

- **Code**:
  - `src/pixoo/error.rs`: generalize `PixooHttpErrorResponse` into the shared envelope (`error_status`, `error_kind`, `message`, plus an optional `details` object skipped when empty) and extend `PixooHttpErrorKind` with `validation`, `not-found`, `payload-too-large`.
  - `src/routes/common.rs`: rewrite `validation_error_simple`, `action_validation_error`, `validation_errors_response`, `service_unavailable`, `internal_server_error` (and retire the `json_error`/`ErrorBuilder` `{error}` path) to emit the envelope.
  - `src/routes/mod.rs`: `not_found()` emits the envelope.
  - `src/routes/draw.rs`: `payload_too_large` and `remote_fetch_failed` emit the envelope.
  - `src/openapi.rs`: collapse `ValidationErrorBody` + `PayloadTooLargeBody` into the single envelope schema referenced everywhere.
- **Tests**: every test asserting `json["error"] == ...` (validation, not-found, payload-too-large) updates to the new fields (`error_kind`, `message`, `details`/`limit`/`actual`).
- **Consumers (BREAKING)**: HTTP clients parsing the old `error` field must switch to `message`/`error_kind`. Documented in README + migration notes.
- **Pixoo constraints**: purely an HTTP response-shape change; no new device interaction.
