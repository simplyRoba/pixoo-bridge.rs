## 1. Setup

- [ ] 1.1 Create feature branch `feat/device-settings-endpoints` (already done)
- [ ] 1.2 Verify project builds and tests pass cleanly before starting

## 2. API Implementation

- [ ] 2.1 Implement `PUT /manage/time/mode/{mode}` handler in `src/api/manage.rs`
  - [ ] Parse `{mode}` (allow only "12h" / "24h")
  - [ ] Invoke generic command `Device/SetTime24Flag` with `Mode: 0` (12h) or `1` (24h)
- [ ] 2.2 Implement `PUT /manage/weather/temperature-unit/{unit}` handler in `src/api/manage.rs`
  - [ ] Parse `{unit}` (allow only "celsius" / "fahrenheit")
  - [ ] Invoke generic command `Device/SetDisTempMode` with `Mode: 0` (C) or `1` (F)
- [ ] 2.3 Wire up new routes in `src/api/server.rs` or module router

## 3. Testing

- [ ] 3.1 Write unit tests for handler logic (valid/invalid inputs)
- [ ] 3.2 Write integration tests mocking the Pixoo client response to verify correct command payloads are sent

## 4. Documentation & Cleanup

- [ ] 4.1 Update `README.md` to document the new `PUT` endpoints
- [ ] 4.2 Run `cargo fmt` and `cargo clippy`
- [ ] 4.3 Run `cargo test` to ensure no regressions
