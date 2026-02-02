# api/manage Capability

## Purpose
Expose read-only GET surfaces under `/manage/*` so automation systems can fetch Pixoo settings, the current clock, and weather data while the bridge handles interpreting Pixoo’s numeric flags and timestamps.

## ADDED Requirements

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
