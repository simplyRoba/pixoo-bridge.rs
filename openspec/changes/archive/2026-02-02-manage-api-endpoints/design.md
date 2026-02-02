## Context

The bridge already hosts `/health`, `/reboot`, and `/tools/*` endpoints on top of a minimal Axum router that shares `AppState { health_forward, pixoo_client: Option<PixooClient> }`. Pixoo interactions happen via the shared `PixooClient::send_command` helper, which enforces retries, backoff, and JSON parsing before returning the parsed payload without `error_code`. Operators now need fast read-only access to the device configuration, clock, and weather without manually composing POST payloads, so we will expose `/manage/settings`, `/manage/time`, and `/manage/weather` as GET endpoints while leaving the Pixoo client, retries, and error handling unchanged.

## Goals / Non-Goals

**Goals:**
- Provide GET `/manage/settings`, `/manage/time`, and `/manage/weather` endpoints behind a single `mount_manage_routes` helper so they can be mounted where the rest of the router lives.
- Forward each request to the corresponding Pixoo command (`Channel/GetAllConf`, `Device/GetDeviceTime`, `Device/GetWeatherInfo`) via `PixooClient::send_command`, log context, and return a typed payload that maps Pixoo flags into documented field names (including display state, time unit, rotation angle, temperature metrics, and ISO-8601 timestamps).
- Surface Pixoo command failures as `503 Service Unavailable` plus a JSON error so integrators understand the bridge is still alive even if the device misbehaves.

**Non-Goals:**
- Caching, aggregation, or transformation beyond returning the Pixoo payloads as-is.
- Converting the GET endpoints into POST requests or modifying existing `/tools/*` proxies.
- Allowing these routes to operate when the Pixoo client is unavailable (they should return `503`).

## Decisions

1. **Add `routes/manage.rs` + `mount_manage_routes`**: mirroring existing `routes/system` and `routes/tools`, the new module keeps manage-specific handlers and wiring in one place and reuses `Extension<Arc<AppState>>` to fetch the optional Pixoo client. Alternative: add the endpoints directly in `main`, but separating the module keeps routing consistent and testable.

2. **Extend `PixooCommand`**: introduce variants for `Channel/GetAllConf`, `Device/GetDeviceTime`, and `Device/GetWeatherInfo` so type-safe callers can declare intent instead of duplicating literal strings. The design keeps `client.send_command` uniform while letting the router describe the command it cares about while keeping the existing retry/backoff behaviour centralized.

3. **Return a typed payload**: each handler calls `state.pixoo_client.send_command(command, Map::new())`, logs the command and timing with `tracing`, converts the Pixoo response into the schema defined in `api/manage` (settings return `displayOn`, `brightness`, `timeMode`, `rotationAngle`, `mirrored`, `temperatureUnit`, `currentClockId`; time returns ISO-8601 `utcTime`/`localTime`; weather returns `weatherString`, `currentTemperature`, `minimalTemperature`, `maximalTemperature`, `pressure`, `humidity`, `windSpeed`), and replies with `Json(payload)`. This transformation keeps the HTTP layer thin while ensuring downstream clients receive normalized data, and it drops unused fields such as `Visibility`.

4. **Error handling mirrors existing routes**: absent Pixoo client or `send_command` failure produce `StatusCode::SERVICE_UNAVAILABLE` plus a `Json({ "error": "Pixoo command failed" })` payload while logging the underlying error. This matches the `/reboot` handler and keeps expectations clear for downstream systems.

## Risks / Trade-offs

- [Risk] Every `/manage/*` request triggers a Pixoo POST, which could overload slow devices or surface retry latency. → **Mitigation**: rely on the existing retry/backoff policy, keep payloads minimal, and return `503` when the device is slow or down rather than timing out indefinitely.
- [Risk] The new GET facade may encourage real-time polling of `Channel/GetAllConf`, which pushes more load onto the bridge than the previous POST-only surfaces. → **Mitigation**: document the endpoints in the `api/manage` spec, including recommended polling cadence and explicit mention that they mirror Pixoo’s normal commands.

## Migration Plan

- Release the next bridge version with the new `/manage/*` routes, update documentation to highlight the GET surfaces and their payload schemas, and let automations switch from POST payloads to the new endpoints without any state migration.

## Open Questions

- Should we gate the `/manage/*` routes behind a feature flag or rate limiter if downstream systems poll too aggressively, or wait until actual pressure surfaces? - NO!
