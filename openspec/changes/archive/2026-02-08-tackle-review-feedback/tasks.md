## 1. Spec-Aligned Configuration Updates

- [x] 1.1 Inventory current startup env loading and document existing `PIXOO_BASE_URL` behavior in code
- [x] 1.2 Update configuration loading to treat missing/invalid `PIXOO_BASE_URL` as a startup error
- [x] 1.3 Load Pixoo client timeout/retry settings once at startup and pass explicit values into client construction

## 2. Pixoo Client Construction Refactor

- [x] 2.1 Replace `Option<PixooClient>` in `AppState` with a concrete client constructed at startup
- [x] 2.2 Remove per-handler `None` guards and adjust handler wiring for the non-optional client

## 3. Testing and Validation

- [x] 3.1 Add unit tests for config parsing (missing/invalid `PIXOO_BASE_URL`, valid configuration)
- [x] 3.2 Add unit tests ensuring client construction uses explicit configuration values
- [x] 3.3 Update/verify any existing tests that assumed optional client behavior

## 4. Docs and Verification

- [x] 4.1 Review README for any required configuration updates and adjust if needed
- [x] 4.2 Run `cargo fmt`, `cargo clippy`, and `cargo test`
