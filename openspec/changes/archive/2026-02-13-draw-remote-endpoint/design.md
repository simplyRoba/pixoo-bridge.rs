## Context

The bridge already exposes `/draw/fill` and `/draw/upload` inside `routes::draw`, relying on `AppState` for Pixoo client access, animation configuration, and `max_image_size`. Uploaded files are validated in `draw_upload`, decoded through `pixels::decode_upload`, and the resulting frames are converted via `encode_pic_data` before being sent through the shared `get_next_pic_id`/`send_draw_frame` helpers that keep Pixoo command sequencing consistent. Adding `/draw/remote` should reuse as much of that shared infrastructure as possible while also adding the smaller client-facing API surface that downloads remote media.

## Goals / Non-Goals

**Goals:**
- Give HTTP clients a `POST /draw/remote` endpoint that accepts `{"link":"http(s)://…"}` payloads, mirrors `/draw/upload` validation, and reuses the image-decoding and Pixoo command helpers so remote images behave exactly like uploads from a quality and limit standpoint.
- Guard the bridge against abusive or broken remote hosts by bounding download size, time, and retries while surfacing actionable error responses when download, validation, or Pixoo sequencing fails.

**Non-Goals:**
- Building a generic remote asset proxy (no caching, transformations, or streaming beyond what the Pixoo animation pipeline already expects).
- Supporting any authentication or stateful interactions with remote hosts beyond unconditional GET + standard URL validation.

## Decisions

- **Dedicated remote fetch client in `AppState`:** Creating a small `reqwest::Client` configured with a low timeout (shared with the Pixoo client via `PIXOO_BRIDGE_REMOTE_TIMEOUT_MS`) and sane connection limits keeps remote downloads simple and separate from Pixoo command traffic. Wrapping that client in a `RemoteFetcher` component makes the download logic reusable, safely scoped, and easy to test.
- **Streaming download with explicit size checks:** Before decoding we must enforce `max_image_size`, so `/draw/remote` will stream the body into a `BytesMut`, aborting once the limit is exceeded. We still honor `Content-Length` when present by rejecting oversized responses upfront, but we also keep counting bytes as we read to protect against hosts lying or trimming chunked responses. This approach mirrors multipart uploads where we already check `bytes.len()` after reading the part, so the Pixoo pipeline sees the same invariant.
- **Reuse the upload-decoding/path to Pixoo commands:** After we have the downloaded bytes and a guessed `content_type`, we call `decode_upload` and the same looping logic that currently feeds `draw_upload`. Any extra metadata (delay scaling, 60-frame cap) already lives there, so reimplementing logic would risk divergence. This keeps `/draw/remote` “thin” (download + validation + hand-off) and avoids introducing new Pixoo command helpers.
- **Error handling surface:** Errors from the remote fetch (timeout, TLS, invalid URI) are mapped to 4xx/5xx responses that resemble the existing validation/internal server error helpers, so clients still receive structured JSON details. Pixoo command failures stay the same as `/draw/upload` by relying on `map_pixoo_error` and the shared helper functions.

## Risks / Trade-offs

- [Remote hosts can return very large files or never finish transmitting] → Mitigate by rejecting responses whose `Content-Length` exceeds `max_image_size`, reading the body into a bounded buffer, and timing out using the dedicated `reqwest::Client` timeout.
- [Remote hosts can be flaky or TLS might fail] → Fail fast with a `503` that includes the underlying HTTP error cause and rely on the shared timeout to bound latency.
- [Guessing the file format from arbitrary links may mis-identify content] → Use the same `content_type` + magic-byte detection logic already exercised by `decode_upload`, so remote fetches fall back to the same success/failure scenarios as uploads.

## Migration Plan

1. Extend `AppState` (and its builder) with a `RemoteFetcher` component backed by a dedicated `reqwest::Client` plus `RemoteFetchConfig` (timeout, max response size). Read the download timeout from `PIXOO_BRIDGE_REMOTE_TIMEOUT_MS`, keep `PIXOO_BRIDGE_MAX_IMAGE_SIZE` shared for both uploads and downloads, and wire the fetcher into `main.rs` so it is available to the draw routes.
2. Register `/draw/remote` in `routes::draw` so it sits beside the existing `/draw` endpoints, and add a new request struct that validates the JSON payload (absolute HTTP or HTTPS URL).
3. Update documentation (README/OpenAPI) and `api/draw` capability descriptions to list the new route, input schema, and failure modes; add integration/unit tests covering successful downloads, oversized responses, and fetch errors.

## Open Questions

- Do we need a separate `PIXOO_REMOTE_MAX_SIZE` environment variable, or can the existing `PIXOO_BRIDGE_MAX_IMAGE_SIZE` suffice for both uploads and remote downloads?
- Would it be useful to expose download latency/size metrics for observability, or is logging fetch failures enough for now?
