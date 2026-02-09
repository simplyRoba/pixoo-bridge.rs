## Context

The bridge exposes a thin HTTP layer over the Pixoo's proprietary API. Today the only draw endpoint is `POST /draw/fill`, which builds a uniform-color 64×64 RGB buffer, base64-encodes it, and sends it via the `Draw/GetHttpGifId` → `Draw/SendHttpGif` command pair. All image construction happens in code — there is no way to push an existing image file to the device.

The `image` crate is the de-facto Rust solution for decoding, resizing, and GIF frame extraction. Axum's `multipart` feature provides streaming multipart form parsing. Neither is currently a dependency.

Key Pixoo constraints:
- Frames are 64×64 RGB, base64-encoded (`PicData`).
- Animations use the same `Draw/SendHttpGif` command, called once per frame with incrementing `PicOffset` (0-indexed) and `PicNum` set to the total frame count.
- Maximum 60 frames per animation (offsets 0–59).
- Frame speed is per-frame in milliseconds (`PicSpeed`).

## Goals / Non-Goals

**Goals:**
- Accept JPEG, PNG, WebP, and GIF uploads via multipart form data and display them on the Pixoo.
- Resize all images to 64×64 to match the panel resolution.
- Support animated GIFs and animated WebP by extracting frames, respecting per-frame delays, and sending multi-frame animations.
- Cap animations at 60 frames, truncating excess.
- Provide a configurable speed factor (`PIXOO_ANIMATION_SPEED_FACTOR`) to scale GIF frame delays.
- Reuse the existing Pixoo command flow and error handling.

**Non-Goals:**
- APNG support (static only for PNG).
- Dithering or color-reduction algorithms for the LED panel.
- Streaming/chunked upload — the entire file is buffered before processing.
- Choosing resize filter quality via API — a single sensible default is used.

## Decisions

### 1. Multipart upload with a single `file` field

Accept `multipart/form-data` with one field named `file`. Axum's `Multipart` extractor handles parsing. The content type is determined from the part's declared content type header, falling back to sniffing magic bytes from the first few bytes of the payload if the content type is missing or `application/octet-stream`.

**Alternative considered:** Raw body with `Content-Type` header. Rejected because multipart is the standard mechanism for file uploads and allows future extension (e.g. additional form fields) without a breaking change.

### 2. Image processing with the `image` crate

Use `image::load_from_memory` for static formats (JPEG, PNG, static WebP) and format-specific decoders for animated formats: `image::codecs::gif::GifDecoder` for GIF and `image::codecs::webp::WebPDecoder` for animated WebP. Both support `into_frames()` for frame extraction. Resize with `image::imageops::resize` using `FilterType::Triangle` (bilinear) — a good balance between quality and speed at 64×64 target size.

Convert each frame to `Rgba8` and then extract the RGB channels into the flat `[R,G,B,…]` buffer that `encode_pic_data` expects. Alpha is composited against a black background before extraction.

**Alternative considered:** `gif` / `webp` crates directly. Rejected because `image` already wraps them and provides a uniform `AnimationDecoder` trait for both formats.

### 3. Animation frame extraction and delay handling

Use `GifDecoder::into_frames()` or `WebPDecoder::into_frames()` (both implement the `AnimationDecoder` trait) to iterate frames. Each `Frame` carries a `Delay` (numerator/denominator in milliseconds). Multiply the delay by `PIXOO_ANIMATION_SPEED_FACTOR` (from `AppState`) and round to the nearest integer for `PicSpeed`. Take at most 60 frames; if the source contains more, log a warning with the original and truncated frame counts using `tracing::warn`.

For static images (including single-frame GIFs/WebPs), send as a single-frame animation with `PicNum=1`, `PicOffset=0`, and the existing `SINGLE_FRAME_PIC_SPEED_MS` constant.

To determine whether a WebP is animated, attempt to construct a `WebPDecoder` and check `has_animation()` before deciding the decode path.

### 4. Configuration

**`PIXOO_ANIMATION_SPEED_FACTOR`** — Add to `AppConfig` as `animation_speed_factor: f64`, parsed from the env var with a default of `1.4`. Invalid or negative values fall back to the default with a warning, matching the existing pattern for `PIXOO_BRIDGE_PORT`.

**`PIXOO_BRIDGE_MAX_IMAGE_SIZE`** — Add to `AppConfig` as `max_image_size: usize` (bytes). Accepts human-readable strings like `5MB`, `128KB`, `1024B` (case-insensitive, with or without the `B` suffix — e.g. `5M` and `5MB` are equivalent). Default is `5MB` (5,242,880 bytes, using binary units: 1 KB = 1024). Invalid values fall back to the default with a warning. Parse with a small helper in `config.rs` rather than pulling in a crate.

Both values are threaded into `AppState` so handlers can access them without re-reading env vars.

### 5. Module layout

- **`src/pixels/imaging.rs`** (new): `decode_upload(bytes, content_type) → Vec<DecodedFrame>` where `DecodedFrame { rgb_buffer: Vec<u8>, delay_ms: u32 }`. Handles format detection, decoding, resizing, alpha compositing, and frame extraction. This keeps image processing logic out of the route handler.
- **`src/pixels/mod.rs`**: Re-export the new module's public API.
- **`src/routes/draw.rs`**: Add the `draw_upload` handler. Reuse the existing `get_next_pic_id` and `send_draw_gif` helpers (already scoped to this module) in a loop over decoded frames.

### 6. Error responses

Follow the existing pattern in `draw.rs`:
- Missing or empty `file` field → `400` with validation error body.
- Upload exceeds `max_image_size` → `413` with limit and actual size in the body.
- Unsupported content type → `400` with `"unsupported image format"`.
- Image decode/resize failure → `400` with `"failed to process image"`.
- Pixoo command failures → existing `map_pixoo_error` flow (502/503/504).

### 7. Upload size enforcement

After reading the multipart `file` field bytes, check the length against `max_image_size` from `AppState`. If the file exceeds the limit, return `413 Payload Too Large` with a JSON body indicating the limit and actual size. The check happens before any image decoding, so oversized uploads don't consume CPU. This limit is stored in `AppState` and can be reused by future image-accepting endpoints.

## Risks / Trade-offs

- **Large upload memory usage** — The source file is buffered in memory before decoding. → Mitigated by the `PIXOO_BRIDGE_MAX_IMAGE_SIZE` limit (default 5 MB) which rejects oversized uploads before any decoding work. Decoded pixel data peaks at ~74 KB (64×64×3 × 60 frames), which is negligible.

- **Blocking image decode on the async runtime** — `image` crate operations are CPU-bound. For the tiny 64×64 target size the work is sub-millisecond, so `spawn_blocking` is not warranted. If profiling shows otherwise, individual decode calls can be moved to `spawn_blocking` later. → Acceptable trade-off for simplicity.

- **Frame disposal/compositing** — The `image` crate's `into_frames()` implementation for both GIF and WebP handles disposal methods (restore to background, combine, etc.) and yields fully-composited RGBA frames. No custom disposal logic is needed. → Low risk.

- **Content-type trust** — We first trust the multipart part's content type, falling back to magic-byte sniffing. A client could label a file incorrectly, but the decoder will fail with a decode error, which maps to a 400 response. → Acceptable.
