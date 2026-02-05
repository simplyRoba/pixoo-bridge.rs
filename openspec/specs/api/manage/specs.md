# api/manage Capability

## Purpose
Expose read-only GET surfaces under `/manage/*` so automation systems can fetch Pixoo settings, the current clock, and weather data while the bridge handles interpreting Pixoo’s numeric flags and timestamps.

## Requirements

### Requirement: Manage weather location command
The bridge SHALL expose `POST /manage/weather/location` so automation systems can tell Pixoo where the device is located. The endpoint SHALL accept a JSON body with `longitude` and `latitude` (float values). Upon receiving valid coordinates, the bridge SHALL send a single `/post` request with `Command: "Sys/LogAndLat"` and the transformed `Longitude`/`Latitude` strings Pixoo expects, reusing the shared retry/backoff helper.

#### Scenario: Setting location forwards to Pixoo
- **WHEN** a client posts `{ "longitude": 30.29, "latitude": 20.58 }` to `/manage/weather/location`
- **THEN** the bridge issues `POST /post` with `{ "Command": "Sys/LogAndLat", "Longitude": "30.29", "Latitude": "20.58" }`, waits for retries, and replies with the Pixoo response body and HTTP 200 once accepted

#### Scenario: Invalid coordinates are rejected
- **WHEN** a client posts an out-of-range coordinate (e.g., longitude `190`) or non-numeric value
- **THEN** the bridge responds with HTTP 400 and a JSON error before calling Pixoo, preventing malformed commands from reaching the device

### Requirement: Manage time zone command
The bridge SHALL expose `POST /manage/time/offset/{offset}` so operators can change the device's timezone offset without crafting Pixoo-specific payloads. The handler SHALL parse `{offset}` as an integer between -12 and +14, format it as `GMT±N`, and issue `POST /post` with `Command: "Sys/TimeZone"` along with the formatted string in `TimeZoneValue`.

#### Scenario: Valid offset applies timezone
- **WHEN** a client sends `POST /manage/time/offset/-5`
- **THEN** the bridge posts `{ "Command": "Sys/TimeZone", "TimeZoneValue": "GMT-5" }` to `/post`, reuses retry helpers, and returns HTTP 200 plus Pixoo's acknowledgement

#### Scenario: Offset out of range fails early
- **WHEN** a client requests an offset outside [-12, 14]
- **THEN** the bridge validates the path parameter, responds with HTTP 400, and does not reach Pixoo so the device never receives an invalid timezone string

#### Scenario: Non-numeric offset is rejected
- **WHEN** a client sends `POST /manage/time/offset/abc`
- **THEN** the bridge rejects the request with HTTP 400 and never issues a Pixoo command because the path parameter cannot be parsed as an integer

### Requirement: Manage device UTC clock command
The bridge SHALL expose `POST /manage/time` so callers can trigger the Pixoo device's UTC clock update without providing a body. The handler SHALL compute the current UTC instant using the system clock, convert it to seconds since the epoch, and then call `/post` with `Command: "Device/SetUTC"` and that computed `Utc` value, mapping Pixoo failures to HTTP 503 after retries are exhausted.

#### Scenario: UTC time is forwarded
- **WHEN** a client calls `POST /manage/time` (no body)
- **THEN** the bridge reads the current UTC timestamp, sends `{ "Command": "Device/SetUTC", "Utc": <current seconds> }` to `/post`, waits for retries, and returns HTTP 200 once Pixoo accepts the update

#### Scenario: System clock cannot be read
- **WHEN** the bridge fails to read the system clock for UTC calculation
- **THEN** the bridge responds with HTTP 500 and does not issue a Pixoo command so clients know the time update could not be attempted

### Requirement: Manage settings endpoint
The bridge SHALL expose `GET /manage/settings` that forwards to `Channel/GetAllConf`, transforms Pixoo’s numeric flags into typed values, and returns only the derived schema so callers do not reimplement the transforms. The response SHALL include:
- `displayOn` (boolean) derived from `LightSwitch == "1"`;
- `brightness` (integer) from `Brightness`;
- `timeMode` (`TWELVE` or `TWENTY_FOUR`) based on whether `Time24Flag == "1"`;
- `rotationAngle` (integer degrees) mapped to `0` when `RotationFlag == "0"` or the flag’s value times `90` otherwise;
- `mirrored` (boolean) from `MirrorFlag == "1"`;
- `temperatureUnit` (`CELSIUS` or `FAHRENHEIT`) based on whether `TemperatureMode == "1"`;
- `currentClockId` (integer) from `CurClockId`.

#### Scenario: Settings payload provides typed values
- **WHEN** a client calls `GET /manage/settings`
- **THEN** the bridge issues `Channel/GetAllConf`, waits for retries, and replies with HTTP 200 plus the derived fields above (no raw Pixoo flags)

#### Scenario: Settings command fails
- **WHEN** the Pixoo client is missing or `Channel/GetAllConf` fails after retries
- **THEN** the bridge returns HTTP 503 with a JSON error describing the failure

### Requirement: Manage time endpoint
The bridge SHALL expose `GET /manage/time` that normalizes Pixoo’s `UTCTime` and `LocalTime` into ISO-8601 timestamps so clients receive consistent datetime values without parsing Pixoo’s formatting. The response SHALL include:
- `utcTime`: ISO-8601 string representing the UTC time calculated from `UTCTime` interpreted as seconds since the epoch in the UTC timezone;
- `localTime`: ISO-8601 string parsed from `LocalTime` using the `yyyy-MM-dd HH:mm:ss` pattern.

#### Scenario: Time payload returns normalized timestamps
- **WHEN** `GET /manage/time` is requested and Pixoo provides `UTCTime`/`LocalTime`
- **THEN** the bridge rewrites the values into ISO-8601 strings, responds with HTTP 200, and logs the conversions

#### Scenario: Time command fails
- **WHEN** the Pixoo client is missing or `Device/GetDeviceTime` errors even after retries
- **THEN** the bridge returns HTTP 503 with a descriptive JSON error

### Requirement: Manage weather endpoint
The bridge SHALL expose `GET /manage/weather` that converts Pixoo’s weather payload into explicitly typed values while dropping `Visibility`. The response SHALL include:
- `weatherString` (string) from Pixoo’s `Weather` (valid values `Sunny`, `Cloudy`, `Rainy`, `Frog`),
- `currentTemperature` (float) from `CurTemp`,
- `minimalTemperature` (float) from `MinTemp`,
- `maximalTemperature` (float) from `MaxTemp`,
- `pressure` (integer) from `Pressure`,
- `humidity` (integer) from `Humidity`,
- `windSpeed` (float) from `WindSpeed`.

#### Scenario: Weather payload provides normalized values
- **WHEN** `GET /manage/weather` is hit and Pixoo returns the weather report
- **THEN** the bridge converts each metric, drops `Visibility`, and responds with HTTP 200 plus the typed fields

#### Scenario: Weather command fails
- **WHEN** the Pixoo client is missing or `Device/GetWeatherInfo` returns an error after retries
- **THEN** the bridge replies with HTTP 503 and a JSON error so callers can retry later

### Requirement: Set time display mode
The bridge SHALL expose `POST /manage/time/mode/{mode}` to allow configuring the device's time format. The endpoint SHALL accept `12h` or `24h` as the `{mode}` path parameter.
- If `{mode}` is `12h`, the bridge SHALL send `POST /post` with `Command: "Device/SetTime24Flag"` and `Mode: 0`.
- If `{mode}` is `24h`, the bridge SHALL send `POST /post` with `Command: "Device/SetTime24Flag"` and `Mode: 1`.

#### Scenario: Set 12-hour mode
- **WHEN** a client sends `POST /manage/time/mode/12h`
- **THEN** the bridge posts `{ "Command": "Device/SetTime24Flag", "Mode": 0 }` to Pixoo and returns HTTP 200 with `{ "error_code": 0 }`

#### Scenario: Set 24-hour mode
- **WHEN** a client sends `POST /manage/time/mode/24h`
- **THEN** the bridge posts `{ "Command": "Device/SetTime24Flag", "Mode": 1 }` to Pixoo and returns HTTP 200 with `{ "error_code": 0 }`

#### Scenario: Invalid time mode
- **WHEN** a client sends `POST /manage/time/mode/invalid`
- **THEN** the bridge returns HTTP 400 and does NOT send a command to Pixoo

### Requirement: Set temperature unit
The bridge SHALL expose `POST /manage/weather/temperature-unit/{unit}` to allow configuring the device's temperature display unit. The endpoint SHALL accept `celsius` or `fahrenheit` as the `{unit}` path parameter.
- If `{unit}` is `celsius`, the bridge SHALL send `POST /post` with `Command: "Device/SetDisTempMode"` and `Mode: 0`.
- If `{unit}` is `fahrenheit`, the bridge SHALL send `POST /post` with `Command: "Device/SetDisTempMode"` and `Mode: 1`.

#### Scenario: Set Celsius
- **WHEN** a client sends `POST /manage/weather/temperature-unit/celsius`
- **THEN** the bridge posts `{ "Command": "Device/SetDisTempMode", "Mode": 0 }` to Pixoo and returns HTTP 200 with `{ "error_code": 0 }`

#### Scenario: Set Fahrenheit
- **WHEN** a client sends `POST /manage/weather/temperature-unit/fahrenheit`
- **THEN** the bridge posts `{ "Command": "Device/SetDisTempMode", "Mode": 1 }` to Pixoo and returns HTTP 200 with `{ "error_code": 0 }`

#### Scenario: Invalid temperature unit
- **WHEN** a client sends `POST /manage/weather/temperature-unit/kelvin`
- **THEN** the bridge returns HTTP 400 and does NOT send a command to Pixoo
