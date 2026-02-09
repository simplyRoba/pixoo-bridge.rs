## Why

The bridge currently only supports programmatic pixel drawing (fill a solid color). Users who already have an image — a logo, photo, or animated GIF — must manually decode it into raw RGB and call the low-level draw API. A file-upload endpoint removes that friction, accepting standard image formats and handling resizing/frame extraction so the Pixoo displays the image immediately.

## What Changes

- Add a `POST /draw/upload` endpoint that accepts multipart file uploads containing JPEG, PNG, WebP, or GIF images.
- Validate the uploaded content type and reject unsupported formats with a structured error.
- Resize static images to 64×64 pixels to match the Pixoo panel resolution.
- For animated GIFs and animated WebP files, extract each frame, resize every frame to 64×64, and send them as a multi-frame animation via the existing `Draw/SendHttpGif` command sequence. The Pixoo supports a maximum of 60 frames (offsets 0–59); files with more frames are truncated to the first 60 with a warning logged.
- Add a `PIXOO_ANIMATION_SPEED_FACTOR` environment variable (default `1.4`) that multiplies the per-frame delay read from the animation file. This lets operators tune animation playback speed on the Pixoo without re-encoding the source file.
- Add a `PIXOO_BRIDGE_MAX_IMAGE_SIZE` environment variable (default `5MB`) that caps the accepted upload size. Accepts human-readable values like `5MB`, `128KB`, etc. Uploads exceeding the limit are rejected before decoding. This limit may be reused by future image-accepting endpoints.
- Add `image` crate dependency for decoding, resizing, and GIF frame extraction.
- Add `axum` multipart feature for file upload handling.

## Capabilities

### New Capabilities
_(none — the upload endpoint extends the existing `api/draw` capability)_

### Modified Capabilities
- `api/draw`: Add the `/draw/upload` endpoint (multipart upload, format validation, image resizing, GIF/WebP animation frame extraction, animation speed factor) and extract the reusable `send_draw_gif` sequencing (get GIF ID → send N frames) as a shared internal helper so both `/draw/fill` and `/draw/upload` use the same Pixoo command flow.
- `core/configuration`: Add `PIXOO_ANIMATION_SPEED_FACTOR` (f64, default `1.4`) and `PIXOO_BRIDGE_MAX_IMAGE_SIZE` (human-readable byte size, default `5MB`) to `AppConfig`, parsed from environment, and threaded into `AppState`.

## Impact

- **Code**: New handler in `src/routes/draw.rs`; new image-processing module under `src/pixels/` (or sibling); extract `get_next_pic_id` / `send_draw_gif` into shared helpers within `draw.rs`. New config field and `AppState` field for the animation speed factor.
- **Dependencies**: `image` crate (decoding + resizing), `axum` multipart feature enabled.
- **API surface**: One new public endpoint (`POST /draw/upload`); no changes to existing endpoints.
- **Pixoo interaction**: Same `Draw/GetHttpGifId` + `Draw/SendHttpGif` command pair, but now with `PicNum > 1` and incremented `PicOffset` for animations.
