## Why

Operators need lightweight ways to read Pixoo device settings, local time, and current weather so that home automation or monitoring systems can stay in sync without reimplementing the unwieldy POST-based CLI. Exposing managed GET endpoints keeps the bridge focused on thin request translation, matches the existing Pixoo client behavior, and unlocks the rest of this capability for automation.

## What Changes

- Add a new `/manage/settings`, `/manage/time`, and `/manage/weather` GET surface that simply forwards to `Channel/GetAllConf`, `Device/GetDeviceTime`, and `Device/GetWeatherInfo`, respectively, returning the payloads the Pixoo device already provides (sans error metadata).
- Keep the Pixoo client and HTTP bridge thin by sending the existing normal POST commands and translating their results verbatim under the `/manage/` prefix; reuse the same routing/mapping patterns already protecting other API facade endpoints.
- Document this capability in a new `api/manage` spec so downstream consumers understand what each endpoint surfaces, including expected numeric flags (e.g., brightness range, rotation/mirror flags, temperature mode, Weather string values).

## Capabilities

### New Capabilities
- `api/manage`: under the `api` domain, describes the `/manage/*` GET endpoints that proxy `Channel/GetAllConf`, `Device/GetDeviceTime`, and `Device/GetWeatherInfo`, including the semantics of each field and how it maps to Pixoo commands.

### Modified Capabilities
- _None_

## Impact

- Adds routing under the API layer (likely `api::router`/`api::handlers`) to mount `/manage/settings`, `/manage/time`, and `/manage/weather` alongside the existing POST proxies.
- Relies on the Pixoo client module to issue the `Channel/GetAllConf`, `Device/GetDeviceTime`, and `Device/GetWeatherInfo` commands; no new low-level transport behavior is required.
- Adds awareness to the specification tree so automation/docs can describe the new GET surfaces and their payloads.
