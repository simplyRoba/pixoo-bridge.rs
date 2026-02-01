## 1. Routing & Pixoo client commands

- [ ] 1.1 Introduce `routes/manage.rs` with handlers for `/manage/settings`, `/manage/time`, and `/manage/weather`, wiring them through `Extension<Arc<AppState>>` and returning typed JSON or `503` on failure.
- [ ] 1.2 Extend `PixooCommand` with `ChannelGetAllConf`, `DeviceGetDeviceTime`, and `DeviceGetWeatherInfo` (and wire each handler to the appropriate variant) so the router can describe the command it issues.

## 2. Payload shaping & tests

- [ ] 2.1 Implement the response transformations defined in `api/manage` (typed settings fields, ISO-8601 timestamps, normalized weather metrics) and log timing/context with tracing for each handler.
- [ ] 2.2 Add focused tests that mock the Pixoo client and assert the new handlers emit the normalized JSON plus convert failures into HTTP 503 responses.

## 3. Documentation & polish

- [ ] 3.1 Update `README.md` or relevant docs so end users know about the new `/manage/*` GET surfaces and their payload schemas.
- [ ] 3.2 Run `cargo fmt`, `cargo clippy`, and `cargo test` before marking the change ready for implementation review.
