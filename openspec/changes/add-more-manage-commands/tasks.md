## 1. Router & models

- [x] 1.1 Define request/response models for `/manage/weather/location`, `/manage/time`, and `/manage/time/offset/{offset}` including validation rules (coordinate ranges, timezone bounds).
- [x] 1.2 Wire the handlers into the existing `api/manage` router so the new POST routes sit beside the read-only `GET` surfaces used today.

## 2. Pixoo command plumbing

- [x] 2.1 Implement or reuse a `send_pixoo_command` helper that serializes a command and payload, hits `/post`, and applies the shared retry/backoff behavior.
- [x] 2.2 Update each handler to translate incoming data (or system UTC time in the case of `/manage/time`) into the Pixoo payloads (`Sys/LogAndLat`, `Sys/TimeZone`, `Device/SetUTC`) sent via the helper, mapping failures to HTTP 503 responses.

## 3. Validation & observability

- [x] 3.1 Enforce validation for longitude/latitude, timezone offset, and UTC timestamp computation, returning HTTP 400 before talking to Pixoo whenever inputs fail.
- [x] 3.2 Log incoming requests plus the translated Pixoo payloads/errors to keep traceability consistent with other manage handlers.

## 4. Tests & docs

- [x] 4.1 Add unit/integration tests that stub the Pixoo client and assert each endpoint issues the correct `/post` payloads under both success and failure scenarios.
- [x] 4.2 Document the new endpoints (paths, payload expectations, validation notes) alongside the existing manage API reference so operators know how to use them.

## 5. Finalization

- [x] 5.1 Run `cargo fmt`, `cargo clippy`, and `cargo test` to ensure the change builds cleanly and matches repo guardrails.
