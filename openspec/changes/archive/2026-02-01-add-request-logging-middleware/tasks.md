## 1. Middleware wiring

- [x] 1.1 Add HTTP logging middleware and any needed dependencies so `build_app` always emits per-request information at DEBUG level without touching handlers.
- [x] 1.2 Update `build_app` in `src/main.rs` so the mounted routes are wrapped by the request-logging middleware while still sharing `Extension(state)`.

## 2. Docs & Verification

- [x] 2.1 Mention the request logging behavior in `README.md`, including that `PIXOO_BRIDGE_LOG_LEVEL=debug` is required for those DEBUG entries.
- [x] 2.2 Run `cargo fmt && cargo clippy && cargo test` to keep the change aligned with repository standards before implementation.
