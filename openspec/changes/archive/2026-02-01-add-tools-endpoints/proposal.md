## Why

The bridge currently exposes only the basic Pixoo control endpoints, but operators also need to drive the sheet music tools (timer, stopwatch, scoreboard, and noise meter) via HTTP so automation systems and dashboards can orchestrate live events without resorting to the device’s unreliable UI.

## What Changes

- Add a `POST /tools/timer` endpoint that accepts the `Tools/SetTimer` payload (minute/second/status) and translates it into the corresponding Pixoo command so callers can start or stop countdowns.
- Add `POST /tools/stopwatch`, `POST /tools/scoreboard`, and `POST /tools/noise` endpoints that accept the documented tool payloads and dispatch `Tools/SetStopWatch`, `Tools/SetScoreBoard`, and `Tools/SetNoiseStatus`, respectively, reusing the existing HTTP-to-Pixoo plumbing and retry helpers.
- Extend the routing layer, request models, and HTTP client helpers so these handlers share the same validation, error mapping, and retry/backoff strategy that already protects other Pixoo commands.

## Capabilities

### New Capabilities
- `api/tools`: Defines the HTTP surface for each Tools command, the required JSON shapes, allowed status codes, and how the bridge sequences retries/backoff when the Pixoo device misbehaves.

### Modified Capabilities
- None.

## Impact

- Adds new API surface (handlers, request models, tests) under the existing routing stack so HTTP clients can trigger timer/stopwatch/scoreboard/noise commands for live control dashboards.
- Relies on the current Pixoo command dispatcher, so the backend HTTP-to-Pixoo layers (serialization, retries, error propagation) must be extended to accept the new request payloads and commands.
- Requires documentation updates and automated tests that assert the new endpoints translate to the correct `Tools/Set*` commands while honoring Pixoo’s flaky API constraints.
