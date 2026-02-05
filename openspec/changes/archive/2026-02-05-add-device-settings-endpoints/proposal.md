## Why

Users currently can view device settings like time format (12h/24h) and temperature unit (C/F) via `GET /manage/settings` but cannot change them via the bridge. This change enables full remote configuration of these display preferences, filling the gap in device management capabilities.

## What Changes

- Add `POST /manage/time/mode/{mode}` endpoint.
  - Accepts `12h` or `24h` as path parameter.
  - Maps to Pixoo command `Device/SetTime24Flag` (sending `0` for 12h, `1` for 24h).
- Add `POST /manage/weather/temperature-unit/{unit}` endpoint.
  - Accepts `celsius` or `fahrenheit` as path parameter.
  - Maps to Pixoo command `Device/SetDisTempMode` (sending `0` for Celsius, `1` for Fahrenheit).
- Ensure both endpoints return the standard `{ "error_code": 0 }` JSON response on success or appropriate error codes on failure.

## Capabilities

### New Capabilities
- None

### Modified Capabilities
- `api/manage`: Add requirements for `POST /manage/time/mode/{mode}` and `POST /manage/weather/temperature-unit/{unit}` to control device display settings.

## Impact

- `api` crate: New route handlers in `manage` module.
- `pixoo-client`: No changes expected if generic command posting is sufficient, otherwise new typed helpers may be added.
