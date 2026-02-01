## Context

The bridge currently exposes only the `/health` and `/reboot` handlers under `routes::system`, and the shared `PixooClient`/`AppState` stack already handles retries, logging, and serialization for those simple commands. Operators now need HTTP-backed control over Pixoo’s timer, stopwatch, scoreboard, and noise tools so automation dashboards can coordinate live shows without touching the physical panel. This change must expand the HTTP surface to relay the documented `Tools/Set*` commands while preserving the disciplined retry, error handling, and dependency constraints that keep the binary small.

## Goals / Non-Goals

- Add `POST /tools/timer/start`, `/tools/timer/stop`, `/tools/stopwatch/{action}`, `/tools/scoreboard`, and `/tools/soundmeter/{action}` so the HTTP surface hides Pixoo’s numeric `Status` values behind descriptive verbs while still translating to the documented command payloads.
- Keep payload processing strongly typed, reuse the existing `PixooClient` retry/backoff helper, and surface consistent success or `503` responses that reflect Pixoo’s flaky HTTP behavior.
- Ensure observability and logging match the rest of the bridge (structured context, Pixoo error codes, retriable flags) and document the new surface in the `api/tools` capability so downstream automation knows how to invoke it.

**Non-Goals:**
- We will not model countdown state, scoreboard totals, or noise thresholds beyond translating the provided fields to Pixoo commands; the bridge remains a stateless proxy.
- No additional third-party dependencies will be introduced just for validation—the response shapes are already small enough for handwritten checks.

## Decisions

### Tool routing surface
Create a new `routes::tools` module that exposes `mount_tool_routes`, mirrors the API style of `routes::system`, and registers the new `POST` routes (`/tools/timer/start`, `/tools/timer/stop`, `/tools/stopwatch/{action}`, `/tools/scoreboard`, `/tools/soundmeter/{action}`). `main::build_app` will call `mount_tool_routes` alongside `mount_system_routes` and pass along the shared `Extension<Arc<AppState>>` so handlers reuse the same state, logging context, and Pixoo client reference without duplicating middleware.

### Typed request models and validation
Define per-endpoint request structs (`TimerRequest`, `ScoreboardRequest`) deriving `serde::Deserialize`. Timer handlers will accept `minute`/`second` from the JSON body and ignore `Status` because the path (start/stop) is the new command trigger; `StopwatchAction` and `SoundmeterAction` path parameters convert to the Pixoo `Status` values (start/stop/reset) before building the payload. Scoreboard requests carry `blue_score`/`red_score` fields constrained to `0..=999`. Perform lightweight validation before building the Pixoo payload so serialization stays explicit and mismatched values are rejected with `400 Bad Request`.

### Pixoo command plumbing
Extend `pixoo::command::PixooCommand` with variants for `ToolsSetTimer`, `ToolsSetStopWatch`, `ToolsSetScoreBoard`, and `ToolsSetNoiseStatus`, each returning the exact command string (`Tools/Set*`). Each handler constructs the `Map<String, Value>` payload from the request struct and passes it to `PixooClient::send_command`, leveraging the existing retry/backoff, error-code parsing, and logging so we do not reimplement those helpers. If the `AppState` does not contain a Pixoo client, handlers fail fast with `503` and a JSON error body.

### Error handling and observability
Handlers mirror the system routes by returning `StatusCode::NO_CONTENT` (or `OK` if we include a status field) upon success and `StatusCode::SERVICE_UNAVAILABLE` with a JSON error when Pixoo rejects the command. Logging will mention the command context and include Pixoo error codes/retriable flags just like `PixooClient::log_pixoo_error`, ensuring operators can correlate issues across endpoints. Any validation errors on the HTTP payload surface as `400 Bad Request` before touching Pixoo so that malformed inputs never consume retry budget.

## Risks / Trade-offs

- [Risk] A noisy automation loop could hammer the Pixoo tool commands and trigger retries, making the bridge appear slow. → Mitigate by keeping the same retry count as existing commands, logging retries, and documenting that callers should respect Pixoo’s soft rate limits.
- [Risk] Scoreboard/stopwatch status fields could drift if clients send invalid numbers. → Validate inputs server-side and reject out-of-range values before dispatching to Pixoo while surfacing clear error messages.
- [Risk] Adding four new handlers increases the bridge’s API surface. → Keep the module focused, rely on the established shared state and client, and centralize command definitions in `pixoo::command` to avoid divergence.

## Migration Plan

1. Update `routes::mount_system_routes` to also call `mount_tool_routes` so the new paths are registered alongside `/health` and `/reboot`.
2. Add request structs, new `PixooCommand` variants, and helper functions that build and dispatch the `Tools/Set*` payloads via the existing `PixooClient` methods.
3. Add unit tests for each handler validating success, Pixoo failure, and missing client scenarios; extend `main` tests to cover at least one tool route.
4. Run `cargo fmt`, `cargo clippy`, and `cargo test` before merging so the Docker image stays small and the new surface stays guarded by linting/test automation.

## Open Questions

- Should we mirror the Pixoo response body (with `error_code`) back to callers on success, or simply return an empty `2xx` and rely on HTTP status?
- Do any of the tool endpoints require authentication/authorization beyond the existing router assumptions, or is the bridge delivered inside a trusted network?
