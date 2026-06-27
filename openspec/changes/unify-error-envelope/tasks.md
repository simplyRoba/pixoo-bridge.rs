## 1. Core envelope type

- [ ] 1.1 Extend `PixooHttpErrorKind` in `src/pixoo/error.rs` with `Validation`, `NotFound`, `PayloadTooLarge` (serde kebab-case: `validation`, `not-found`, `payload-too-large`)
- [ ] 1.2 Add `details: Option<serde_json::Value>` to `PixooHttpErrorResponse` with `#[serde(skip_serializing_if = "Option::is_none")]` and a documented schema note
- [ ] 1.3 Add constructors: `new(status, kind, message)` (no details) and `with_details(status, kind, message, details)`; keep the coherent struct-level `#[schema(example = ...)]`
- [ ] 1.4 Ensure `map_pixoo_error` still produces device kinds and sets `details = { "error_code": .. }` only when a code is present

## 2. Route error producers

- [ ] 2.1 Rewrite `validation_error_simple`, `action_validation_error`, and `validation_errors_response` in `src/routes/common.rs` to emit the envelope (`error_kind = validation`, per-field/action map under `details`)
- [ ] 2.2 Rewrite `service_unavailable` and `internal_server_error` in `src/routes/common.rs` to use the envelope constructors
- [ ] 2.3 Rewrite `not_found` in `src/routes/mod.rs` to emit the envelope (`error_kind = not-found`, no details)
- [ ] 2.4 Rewrite `payload_too_large` (`error_kind = payload-too-large`, `details = { limit, actual }`) and `remote_fetch_failed` (`error_kind = remote-fetch`) in `src/routes/draw.rs`
- [ ] 2.5 Remove `json_error`/`ErrorBuilder` from `src/routes/common.rs` once unused, and fix any remaining references

## 3. OpenAPI documentation

- [ ] 3.1 Remove `ValidationErrorBody` and `PayloadTooLargeBody` from `src/openapi.rs`; keep only the canonical envelope schema in `components`
- [ ] 3.2 Update every `#[utoipa::path(...)]` error response across draw/tools/system/manage to `body = PixooHttpErrorResponse` (400/404/413/500/502/503/504)
- [ ] 3.3 Verify the generated spec: all error responses reference one schema and the example is coherent

## 4. Tests

- [ ] 4.1 Update all tests asserting the old `json["error"]`/root `limit`/`actual`/`error_code` to the new envelope (`error_kind`, `message`, nested `details`) across routes and `main.rs`
- [ ] 4.2 Add/adjust a test asserting a 400 validation body has root `error_status`/`error_kind`/`message` and `details` with the field errors
- [ ] 4.3 Add/adjust a test asserting a 413 body nests `limit`/`actual` under `details` (not at root)
- [ ] 4.4 Add/adjust a test asserting the 404 fallback body is exactly `{ error_status, error_kind: "not-found", message }` with no `details`

## 5. Documentation & verification

- [ ] 5.1 Update `README.md` error section and the Kotlin migration notes to describe the unified envelope (BREAKING: `error` removed, extras nested under `details`)
- [ ] 5.2 Run `cargo fmt`, `cargo clippy`, and `cargo test`; resolve all warnings and failures
