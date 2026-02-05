## 1. Setup

- [x] 1.1 Create feature branch `feat/device-settings-endpoints` (already done)
- [x] 1.2 Verify project builds and tests pass cleanly before starting

## 2. API Implementation

- [x] 2.1 Implement `PUT /manage/time/mode/{mode}` handler in `src/routes/manage.rs`
  - [x] Parse `{mode}` (allow only "12h" / "24h")
  - [x] Invoke generic command `Device/SetTime24Flag` with `Mode: 0` (12h) or `1` (24h)
- [x] 2.2 Implement `PUT /manage/weather/temperature-unit/{unit}` handler in `src/routes/manage.rs`
  - [x] Parse `{unit}` (allow only "celsius" / "fahrenheit")
  - [x] Invoke generic command `Device/SetDisTempMode` with `Mode: 0` (C) or `1` (F)
- [x] 2.3 Wire up new routes in `src/routes/manage.rs` or module router

## 3. Testing

- [x] 3.1 Write unit tests for handler logic (valid/invalid inputs)
- [x] 3.2 Write integration tests mocking the Pixoo client response to verify correct command payloads are sent

## 4. Documentation & Cleanup

- [x] 4.1 Update `README.md` to document the new `PUT` endpoints
- [x] 4.2 Run `cargo fmt` and `cargo clippy`
- [x] 4.3 Run `cargo test` to ensure no regressions
