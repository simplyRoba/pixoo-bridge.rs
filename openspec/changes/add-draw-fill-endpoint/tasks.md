## 1. Draw route prep

- [x] 1.1 Add a new `/draw/fill` Axum route and request struct with validator-based RGB range checks.
- [x] 1.2 Hook the route into the HTTP router and ensure it returns structured error responses for invalid payloads.

## 2. Pixoo command plumbing

- [x] 2.1 Extend `PixooCommand` and related command handling to include `Draw/GetHttpGifId`, `Draw/SendHttpGif`, and `Draw/ResetHttpGifId`.
- [x] 2.2 Reuse `PixooClient::send_command` so the draw commands benefit from existing retries/backoff and error mapping.

## 3. Pixel payload helper

- [x] 3.1 Implement the reusable helper that emits Base64 `PicData` for a 64×64 pixel buffer (row-major `[r,g,b]` bytes).
- [x] 3.2 Write unit tests verifying the helper against uniform colors and key edge cases.

## 4. Draw fill implementation and tests

- [x] 4.1 Implement the fill handler: generate the uniform buffer, fetch a PicID, build the automation payload, and send it to Pixoo.
- [x] 4.2 Add remote tests (mock Pixoo server) to assert the command sequence and payload contents for a successful fill.

## 5. Wrap-up

- [x] 5.1 Document the new endpoint and payload expectations in the README or API docs if needed.
- [x] 5.2 Run `cargo fmt`, `cargo clippy`, and `cargo test`, then note their success for verification.
