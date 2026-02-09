## 1. Setup

- [x] 1.1 Create feature branch `feat/draw-upload-endpoint` off main
- [x] 1.2 Add `image` crate dependency to `Cargo.toml`
- [x] 1.3 Enable `multipart` feature on the `axum` dependency in `Cargo.toml`

## 2. Configuration

- [x] 2.1 Add human-readable byte-size parser helper to `src/config.rs` (e.g. `5MB`, `128K` → `usize`) with unit tests
- [x] 2.2 Add `PIXOO_ANIMATION_SPEED_FACTOR` (f64, default 1.4) to `AppConfig` with fallback on invalid/non-positive values and unit tests
- [x] 2.3 Add `PIXOO_BRIDGE_MAX_IMAGE_SIZE` (human-readable, default 5MB) to `AppConfig` with fallback on invalid values and unit tests
- [x] 2.4 Add `animation_speed_factor: f64` and `max_image_size: usize` fields to `AppState` and wire them from `AppConfig` in `main.rs`

## 3. Image processing module

- [x] 3.1 Create `src/pixels/imaging.rs` with `DecodedFrame` struct (`rgb_buffer: Vec<u8>`, `delay_ms: u32`) and format detection logic (content type header with magic-byte fallback)
- [x] 3.2 Implement static image decoding (JPEG, PNG, static WebP): decode, resize to 64×64, composite alpha against black, return single `DecodedFrame`
- [x] 3.3 Implement animated GIF decoding: extract frames via `GifDecoder::into_frames()`, resize each to 64×64, composite alpha, read per-frame delay, cap at 60 frames with `tracing::warn` on truncation
- [x] 3.4 Implement animated WebP decoding: detect via `WebPDecoder::has_animation()`, extract frames via `into_frames()`, same resize/alpha/delay/cap logic as GIF
- [x] 3.5 Re-export public API from `src/pixels/mod.rs`
- [x] 3.6 Write unit tests for imaging module: static decode (JPEG/PNG/WebP), animated GIF multi-frame, animated WebP multi-frame, 60-frame truncation, unsupported format rejection, alpha compositing

## 4. Draw route handler

- [x] 4.1 Extract `get_next_pic_id` and `send_draw_gif` in `src/routes/draw.rs` into shared helpers usable by both `draw_fill` and the new `draw_upload` handler (refactor, no behavior change)
- [x] 4.2 Implement `draw_upload` handler: multipart extraction, file field validation, size check against `max_image_size` (413 on exceed), format validation, call imaging module, loop frames through shared Pixoo command helpers with speed factor applied to delays
- [x] 4.3 Mount `POST /draw/upload` route in `mount_draw_routes`
- [x] 4.4 Write tests for `draw_upload`: successful static image upload, successful animated GIF upload (multi-frame Pixoo command sequence), missing file field (400), empty file field (400), oversized upload (413), unsupported format (400)

## 5. Verification

- [x] 5.1 Run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test` — fix any issues
- [x] 5.2 Update README.md with `/draw/upload` endpoint documentation and new environment variables (`PIXOO_ANIMATION_SPEED_FACTOR`, `PIXOO_BRIDGE_MAX_IMAGE_SIZE`)
