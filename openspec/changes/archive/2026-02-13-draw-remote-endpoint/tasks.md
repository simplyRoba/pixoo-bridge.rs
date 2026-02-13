## 1. Remote fetch infrastructure

- [x] 1.1 Implement a `RemoteFetcher` component that owns a dedicated `reqwest::Client`, honors `PIXOO_BRIDGE_REMOTE_TIMEOUT_MS`, enforces `PIXOO_BRIDGE_MAX_IMAGE_SIZE` during downloads, and exposes a method that returns downloaded bytes plus a guessed content type (no retries).
- [x] 1.2 Wire the fetcher into `AppState` (including its config loader) so the draw routes can borrow it separately from the Pixoo client.

## 2. Draw remote route

- [x] 2.1 Add a validated request struct for `{ "link": "http(s)://…" }` and mount `/draw/remote` in `mount_draw_routes`, ensuring it only accepts absolute HTTP/HTTPS URIs.
- [x] 2.2 Implement the handler that calls `RemoteFetcher`, honors size/time limits, reuses `decode_upload`/Pixoo helpers, and maps validation/download errors to the appropriate HTTP responses described in the spec.

## 3. Docs & tests

- [x] 3.1 Write tests covering a successful remote PNG download path, rejection when the remote payload exceeds the configured size, and failures triggered by invalid URLs or download errors.
- [x] 3.2 Update README/HTTP docs so `/draw/remote` is documented alongside the existing draw endpoints (no deep implementation details, just the new payload and failure conditions).

## 4. Verification

- [x] 4.1 Run `cargo fmt`, `cargo clippy`, and `cargo test` to confirm the repository is in a clean, tested state before archiving the change.
