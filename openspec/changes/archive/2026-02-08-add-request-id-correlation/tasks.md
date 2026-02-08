## 1. Middleware & Request-ID plumbing

- [x] 1.1 Define a `RequestId` newtype, helpers for generating/retrieving it, and a shared `RequestIdHeader` constant so the middleware and handlers agree on the header name.
- [x] 1.2 Implement Tower middleware that reads or generates the request id, stores it in the request extensions, and ensures the response echoes `X-Request-Id`, then wire it into the main router so every incoming request passes through it.

## 2. Logging & observability integration

- [x] 2.1 Annotate Axum handlers/dispatch helpers with `#[tracing::instrument(skip(state))]` (or equivalent manual span creation) so spans inherit the request id and log entries pick up the field.
- [x] 2.2 Update `map_pixoo_error`/logging helpers and `PixooClient::send_command` so every log entry they emit includes the current request id when available.
- [x] 2.3 Ensure Pixoo command invocations carry the identifier (e.g., by passing it through `dispatch_command`/`dispatch_manage_post_command`) so retries and errors log the correct id.

## 3. Tests & documentation

- [x] 3.1 Add tests covering the new middleware (response header, extensions value) and `RequestId` helpers.
- [x] 3.2 Update `README.md` (or relevant docs) to mention the new `X-Request-Id` header and how to correlate logs.
- [x] 3.3 Run `cargo fmt`, `cargo clippy`, and `cargo test` after implementation to prove the change is ready for review.
