# Tasks: Clippy Pedantic as Project Standard

## Completed

- [x] Add `[lints.clippy]` section to `Cargo.toml` with pedantic enabled
- [x] Configure allowed exceptions for noisy lints
- [x] Fix `as` casts in `src/pixoo/client.rs` (use `try_from`)
- [x] Fix let-else patterns in `src/routes/system.rs`
- [x] Fix let-else patterns in `src/routes/manage.rs`
- [x] Fix let-else patterns in `src/routes/tools.rs`
- [x] Fix pass-by-reference in `validation_errors_response`
- [x] Fix match arm consolidation in `src/pixoo/error.rs`
- [x] Fix wildcard pattern in `src/pixoo/error.rs`
- [x] Fix `map_or_else` usage in `client_timeout()`
- [x] Add `#[allow(clippy::type_complexity)]` for test with complex state type
- [x] Verify all tests pass
- [x] Verify `cargo clippy --all-targets -- -D warnings` passes
