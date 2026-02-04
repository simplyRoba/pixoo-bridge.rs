# Tasks: Clippy Pedantic as Project Standard

## 1. Configuration

- [x] 1.1 Add `[lints.clippy]` section to `Cargo.toml` with pedantic enabled
- [x] 1.2 Configure `must_use_candidate = "allow"`
- [x] 1.3 Configure `module_name_repetitions = "allow"`

## 2. Fix pixoo module

- [x] 2.1 Fix `as` casts in `src/pixoo/client.rs` (use `try_from`)
- [x] 2.2 Fix `map_or_else` usage in `client_timeout()`
- [x] 2.3 Fix match arm consolidation in `src/pixoo/error.rs`
- [x] 2.4 Fix wildcard pattern in `src/pixoo/error.rs`
- [x] 2.5 Add `#[allow(clippy::type_complexity)]` for test with complex state type

## 3. Fix routes module

- [x] 3.1 Fix let-else patterns in `src/routes/system.rs`
- [x] 3.2 Fix let-else patterns in `src/routes/manage.rs`
- [x] 3.3 Fix let-else patterns in `src/routes/tools.rs`
- [x] 3.4 Fix pass-by-reference in `validation_errors_response`

## 4. Add error documentation

- [x] 4.1 Add `# Errors` docs to `PixooClient::new`
- [x] 4.2 Add `# Errors` docs to `PixooClient::send_command`
- [x] 4.3 Add `# Errors` docs to `PixooClient::health_check`

## 5. Verify

- [x] 5.1 Verify all tests pass
- [x] 5.2 Verify `cargo clippy --all-targets -- -D warnings` passes
