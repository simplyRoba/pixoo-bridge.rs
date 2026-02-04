## Context

The bridge already centralizes `GET /manage/*` routes that normalize Pixoo’s settings, time, and weather payloads so automation systems can consume typed data without reimplementing Pixoo’s command syntax. The device only accepts state changes via `POST /post`, the Pixoo client library already offers retry/backoff helpers, and the HTTP layer is deliberately thin to keep the Docker image minimal. This design therefore focuses on layering a small number of new POST handlers over the existing manage domain while reusing the Pixoo command executor and adhering to the project’s robustness requirements.

## Goals / Non-Goals

- **Goals:**
- Expose `POST /weather/location`, `POST /time`, and `POST /time/offset/{offset}` within the `api/manage` domain so callers can declare location, UTC clock, and timezone offset in a single surface.
- Translate each request into the corresponding Pixoo command (`Sys/LogAndLat`, `Device/SetUTC`, `Sys/TimeZone`) sent via the existing `/post` route while reusing the shared retry/backoff helper and logging.
- Validate request data (coordinates, UNIX timestamp, timezone offset) through strongly typed request models to keep HTTP interaction explicit and avoid silent command failures.

**Non-Goals:**
- Replacing or broadening the manage capability beyond the new POST endpoints (the GET surfaces remain untouched).
- Introducing additional external dependencies; the change must retain the lightweight Rust binary and reuse existing infrastructure modules.

## Decisions

1. **Route placement & handler shape:** The new POST routes live next to the existing `/manage` handlers to keep all manage controls in one router. Each handler will deserialize a typed request struct (e.g., `SetLocation { longitude: f64, latitude: f64 }`) via the existing Axum request extractors, perform validation, and then call a shared helper that sends the Pixoo command via `/post`.
2. **Local UTC computation:** The `/manage/time` handler does not accept a body; it simply reads the current UTC clock using `SystemTime::now()`, converts it to seconds since the epoch, and passes that value to `Device/SetUTC`. This keeps the API idempotent for callers and avoids extra framing for simple cases.
3. **Command executor reuse:** Instead of crafting bespoke HTTP clients, every handler delegates to the existing Pixoo command helper that already applies retries and logs errors. A new `send_pixoo_command(command: &str, payload: impl Serialize)` utility will wrap the current `/post` verb, so we avoid repeating serialization logic and centralize error handling (mapping failures to HTTP 503 when retries are exhausted).
4. **Validation strategy:** Coordinate payloads will ensure `longitude`/`latitude` stay within valid ranges ([-180, 180], [-90, 90]) before submitting to Pixoo. The timezone offset handler uses a path parameter validated against `^([+-]?\d{1,2})$` and clamps to [-12, 14] to match Pixoo’s expected `GMT±N` strings; the handler formats the string as `GMT{sign}{abs(offset)}` before issuing `Sys/TimeZone`. UTC timestamps are required to be non-negative integers so we do not send dates before 1970.
5. **Logging & observability:** Each handler logs the incoming command plus the translated Pixoo message to maintain traceability. Error paths emit descriptive messages before returning HTTP 503, matching the manage GET endpoints’ behavior.
6. **Docs & tests:** Document the new endpoints (bodies and expected responses) alongside existing manage API docs. Add unit/integration tests that stub the Pixoo client and assert that the correct command payloads reach `/post`, including validation failures.

## Risks / Trade-offs

[Risk] → Pixoo’s `/post` endpoint may silently drop commands if payload validation fails. **Mitigation:** enforce strict validation client-side, return 400 for invalid input, and rely on retries for transient failures.
[Risk] → The timezone offset path parameter might be misinterpreted by clients (e.g., `-05` vs `GMT-5`). **Mitigation:** document the `offset` as an integer with expected range and format, and normalize it to `GMT±N` before sending.
[Risk] → Setting location/time triggers immediately visible changes; mis-specified values could confuse automation. **Mitigation:** log the command and keep the API surface idempotent by always returning the underlying Pixoo response (without caching state) so callers can confirm success.

## Migration Plan

1. Extend the manage router with the three POST routes and share request/response models in the existing API module.
2. Implement the `send_pixoo_command` helper and wire handlers to call it with the correct `Command`/payload combination; reuse the retry/backoff helper already used for GET requests.
3. Add validation unit tests for request deserialization plus integration tests that mock the Pixoo client to verify the `/post` payload and error handling.
4. Update API documentation to describe the new endpoints, including sample requests and the required timezone formatting.
5. Run `cargo fmt`, `cargo clippy`, and `cargo test` locally before submitting the change.

## Open Questions

- Should the timezone offset handler accept minute-resolution offsets (e.g., `GMT+5:30`), or is the integer-based range sufficient given Pixoo’s limitations? Current plan is to keep it integer-based for simplicity.
- Do we need to persist anything about the requested timezone/location in the bridge, or is forwarding the command once enough for the downstream operators? Just forwarding commands (no cached state) seems acceptable.
