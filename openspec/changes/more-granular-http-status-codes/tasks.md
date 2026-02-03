## 1. Error classification core work

- [x] 1.1 Extend the Pixoo client error type to surface reachable/timeout/device failure metadata that the HTTP layer can inspect.
- [x] 1.2 Implement `core::http_error_mapping` that converts those error categories into HTTP 502/503/504 plus a structured JSON body.

## 2. Endpoint integration and routing

- [x] 2.1 Update `/health`, `/manage/*`, `/reboot`, and other Pixoo-facing handlers to call the shared mapper before sending error responses.
- [x] 2.2 Ensure each handler continues to surface relevant fields (e.g., `error_code`) in the JSON payload so operators still see the underlying Pixoo diagnostics.

## 3. Verification and release prep

- [x] 3.1 Add unit/tests asserting the mapper returns the expected status for unreachable, timed-out, and device-error scenarios.
- [x] 3.2 Refresh README.md usage/monitoring guidance to mention the new `core/http-error-mapping` capability and status distinctions.
- [x] 3.3 Run `cargo fmt`, `cargo clippy`, and `cargo test` before archiving the change.
