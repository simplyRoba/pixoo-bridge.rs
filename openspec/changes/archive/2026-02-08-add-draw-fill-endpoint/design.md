## Context

Pixoo already exposes a number of manage and system endpoints through `src/routes/manage.rs`, but there is no draw-specific API yet. To send custom images we need to go through the proprietary `Draw/SendHttpGif` command, which requires a valid animation ID plus Base64-encoded 64×64 pixel data. The new `/draw/fill` endpoint will bootstrap this flow, keep the HTTP layer thin, and surface reusable helpers so future draw automation routes share consistent payload generation and Pixoo command sequencing.

## Goals / Non-Goals

**Goals:**
- Add a `/draw/fill` POST handler that accepts `{ red, green, blue }`, enforces 0–255 bounds, and issues the Pixoo commands (`Draw/GetHttpGifId`, then `Draw/SendHttpGif`) required to fill the display with a single-color frame.
- Keep the Pixoo command surface, client wiring, and retry behavior consistent with existing manage routes while supporting the new draw payloads.
- Provide reusable utilities for constructing 64×64 pixel buffers and converting them to the Base64 `PicData` format expected by Pixoo, so future draw endpoints can reuse the same helper.

**Non-Goals:**
- Supporting anything beyond 64×64 frames for now (Pixoo only advertises 64-wide matrices).
- Adding an open API for managing `PicID` values; the handler will always fetch the next ID internally.
- Implementing multi-frame animations in this change—we only need one frame for the fill use case.

## Decisions

- **Route location and validation:** The new endpoint belongs under a dedicated `routes/draw.rs` (or similar) so the rest of the API remains organized by capability. Input validation (0–255) will reuse `validator` patterns already in the repo and reject invalid RGB values with structured error responses.
- **PixooCommand expansion:** Extend `PixooCommand` with `DrawSendHttpGif`, `DrawGetHttpGifId`, and `DrawResetHttpGifId` variants so we can reuse the command builder and logging already defined in `PixooClient`. The client will continue to wrap every command with retries/backoff, keeping the HTTP layer unchanged.
- **Base64 helper reuse:** Implement a helper (likely near the Pixoo client or a new `pixoo::draw` module) that takes a `[[r,g,b]]` representation and produces the Base64 `PicData` string following the Kotlin logic (pixels ordered left-to-right, top-to-bottom, with each color channel emitting a byte). The fill endpoint builds a uniform buffer and feeds it through this helper—subsequent draw endpoints can invoke the same helper with different data.
- **Command sequence:** The handler will fetch a PicID (via `Draw/GetHttpGifId`), craft a single-frame automation payload (PicNum=1, PicOffset=0, PicWidth=64, PicSpeed=0 or small), and send it with `Draw/SendHttpGif`. Error handling should reuse `map_pixoo_error` so clients get consistent HTTP status codes when Pixoo fails, and logging will reference the new command names.
- **Threading/Async considerations:** The entire command sequence will run sequentially within the handler; no additional concurrency is required as the Pixoo client already holds shared state and handles in-flight requests safely.

## Risks / Trade-offs

- [Pixoo API flakiness] → Even fetching the PicID can fail; rely on the existing retry/backoff and surface failures as 503 so callers know to retry. Consider adding circuit-breaker behavior later if failures spike.
- [Single-frame assumption] → Future draw endpoints may require multi-frame animations; designing the Base64 helper now keeps us ready, but the current implementation will only emit the one-color buffer, so any future multi-frame code will need to build a sequence of buffers before calling `Draw/SendHttpGif` repeatedly.
- [PicID collision] → The device expects ever-increasing PicIDs. We always fetch a fresh ID right before sending to minimize collisions, but rapid successive calls could still produce high values. We will track the last ID in logs for diagnostics if failures occur.
- [Payload size] → A 64×64 frame is sizeable; ensure Base64 generation is efficient (no unnecessary allocations) and reuse buffers when possible to avoid per-request heap churn.

## Migration Plan

1. Add the draw route, request type, and validation tests; extend routing to mount `/draw/fill` alongside existing routes.
2. Update `PixooCommand` and any helpers to cover the draw commands, ensuring logging and error mapping remain consistent.
3. Implement the Base64 helper and unit tests for the color-to-byte conversion (reuse fixtures or sampled values where possible). 4. Write integration tests that stub the Pixoo HTTP endpoint to verify the command sequence (Get ID → Send GIF) and the generated payload.
5. Once tests and documentation pass, merge and publish as part of the next release; no database migrations or external dependencies are affected.

## Open Questions

- Should the Base64 helper live next to the client or in a dedicated `pixoo::draw` module so future automation builders can re-use it without pulling in Pixoo client internals? - own module seems cleaner!
- Do we need to expose metrics (counters) or telemetry for draw endpoint invocations to monitor PicID errors later? - no.
