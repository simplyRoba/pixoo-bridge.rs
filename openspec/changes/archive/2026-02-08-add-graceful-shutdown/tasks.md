## 1. Implementation

- [x] 1.1 Create `shutdown_signal()` async function that awaits SIGTERM or SIGINT using `tokio::signal`
- [x] 1.2 Add `#[cfg(unix)]` conditional for SIGTERM handling (compile-time platform support)
- [x] 1.3 Wire `with_graceful_shutdown(shutdown_signal())` into `axum::serve` call in main
- [x] 1.4 Add INFO log when shutdown signal is received
- [x] 1.5 Add INFO log when shutdown completes

## 2. Verification

- [x] 2.1 Run `cargo fmt`, `cargo clippy`, and `cargo test`
- [x] 2.2 Manual test: start server, send SIGTERM, verify clean exit and logs
