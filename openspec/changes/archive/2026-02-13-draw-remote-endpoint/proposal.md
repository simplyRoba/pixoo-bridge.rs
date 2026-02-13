## Why

Pixoo clients already upload animated assets through `/draw/upload`, but automation scripts sometimes already host their image on a public URL. Hitting that endpoint still requires the caller to download and re-upload, which adds latency and doubles bandwidth. Providing a JSON-driven `/draw/remote` that fetches the image itself reduces friction for HTTP clients and centralizes decoding, size checks, and Pixoo sequencing, which also unlocks better telemetry around remote fetch failures.

## What Changes

- Add a `POST /draw/remote` handler that accepts `{ "link": "http(s)://…" }`, validates the URI, downloads the payload, and reuses the existing image-processing pipeline so remote assets get normalized, resized, and framed exactly like `/draw/upload`.
- Share the existing size, format, and animation limits with the new endpoint, but enforce them before decoding the downloaded bytes while keeping the download flow simple and retry-free.
- Surface the new endpoint through the HTTP router, update OpenAPI/docs, and to keep Pixoo sequencing consistent reuse the shared GIF ID and frame sending helpers already employed by `/draw/fill` and `/draw/upload`.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `api-draw`: Extend the draw capability to include a `POST /draw/remote` endpoint that downloads an image from a provided URL, mirrors the upload validation/processing semantics, and funnels the resulting frames through the shared Pixoo command helpers.

## Impact

- Adds a new HTTP route and handler under `api::draw` (and any routing tables) and hooks it into the same image-processing helpers and Pixoo command helpers already exercised by `/draw/upload`.
- Requires HTTP client configuration (timeout, max response size) so the handler can download remote assets safely; the download client does not perform retries.
- Updates documentation or API specs to advertise `/draw/remote` and ensures the new path is covered by existing integration tests or adds new ones where needed.
