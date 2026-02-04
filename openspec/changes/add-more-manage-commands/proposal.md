## Why

Operators currently cannot configure the Pixoo device’s location, timezone, or clock through the bridge, which makes weather calculations and timestamp conversions brittle for installations that move between zones or need a precise UTC seed. Adding dedicated endpoints gives automation systems an “intent” surface where they can push those settings once and have the bridge translate them into the proprietary Pixoo commands.

## What Changes

- Add POST `/weather/location`, `/time`, and `/time/offset/{offset}` so callers can declare where the device lives, what UTC timestamp it should report, and which timezone offset to use.
- Map each endpoint to the existing Pixoo `/post` command path, translating request details into `Sys/LogAndLat`, `Device/SetUTC`, and `Sys/TimeZone` payloads respectively, including validation for the numeric coordinates and timezone strings and computing the current UTC timestamp server-side so `/manage/time` requires no request body.
- Document the new routes and wiring so operators understand they use the same HTTP client as the other management endpoints.

## Capabilities

### New Capabilities
- None

### Modified Capabilities
- `api/manage`: Add POST endpoints so operators can set `Sys/LogAndLat`, `Device/SetUTC`, and `Sys/TimeZone` over `/post`, extending the existing manage surface rather than creating a standalone capability.

## Impact

- HTTP routing: add handlers for the new `/weather/location`, `/time`, and `/time/offset/{offset}` paths and wire them into the manage middleware.
- Pixoo command layer: build translators that serialize our typed payloads into JSON bodies the device accepts via `/post` and reuse the existing retry/backoff helpers.
- Tests and docs: add integration/unit coverage for the new endpoints and describe the new commands in the README or API reference so operators know which payloads to send.
