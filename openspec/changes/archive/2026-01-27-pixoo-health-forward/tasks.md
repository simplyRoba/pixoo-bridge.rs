## 1. Configuration and API wiring

- [x] 1.1 Add `PIXOO_BRIDGE_HEALTH_FORWARD` config parsing with default true
- [x] 1.2 Expose `GET /health` route in the bridge HTTP router
- [x] 1.3 Implement handler response shape `{ "status": "ok" }` on success

## 2. Pixoo client health check

- [x] 2.1 Add Pixoo client method for GET `/get` health check
- [x] 2.2 Reuse existing retry/backoff policy for the health call
- [x] 2.3 Return unhealthy on non-200 or invalid JSON response

## 3. Health forwarding behavior

- [x] 3.1 Gate forwarding on `PIXOO_BRIDGE_HEALTH_FORWARD` (short-circuit to healthy when false)
- [x] 3.2 Map Pixoo health failures to HTTP 503
- [x] 3.3 Add unit tests for health endpoint and forwarding toggle

## 4. Docs and verification

- [x] 4.1 Update README.md with `/health` usage and `PIXOO_BRIDGE_HEALTH_FORWARD`
- [x] 4.2 Run `cargo fmt`, `cargo clippy`, and `cargo test`
