## ADDED Requirements

### Requirement: Manage weather location command
The bridge SHALL expose `POST /manage/weather/location` so automation systems can tell Pixoo where the device is located. The endpoint SHALL accept a JSON body with `longitude` and `latitude` (float values). Upon receiving valid coordinates, the bridge SHALL send a single `/post` request with `Command: "Sys/LogAndLat"` and the transformed `Longitude`/`Latitude` strings Pixoo expects, reusing the shared retry/backoff helper.

#### Scenario: Setting location forwards to Pixoo
- **WHEN** a client posts `{ "longitude": 30.29, "latitude": 20.58 }` to `/manage/weather/location`
- **THEN** the bridge issues `POST /post` with `{ "Command": "Sys/LogAndLat", "Longitude": "30.29", "Latitude": "20.58" }`, waits for retries, and replies with the Pixoo response body and HTTP 200 once accepted

#### Scenario: Invalid coordinates are rejected
- **WHEN** a client posts an out-of-range coordinate (e.g., longitude `190`) or non-numeric value
- **THEN** the bridge responds with HTTP 400 and a JSON error before calling Pixoo, preventing malformed commands from reaching the device

### Requirement: Manage time zone command
The bridge SHALL expose `POST /manage/time/offset/{offset}` so operators can change the device’s timezone offset without crafting Pixoo-specific payloads. The handler SHALL parse `{offset}` as an integer between -12 and +14, format it as `GMT±N`, and issue `POST /post` with `Command: "Sys/TimeZone"` along with the formatted string in `TimeZoneValue`.

#### Scenario: Valid offset applies timezone
- **WHEN** a client sends `POST /manage/time/offset/-5`
- **THEN** the bridge posts `{ "Command": "Sys/TimeZone", "TimeZoneValue": "GMT-5" }` to `/post`, reuses retry helpers, and returns HTTP 200 plus Pixoo’s acknowledgement

#### Scenario: Offset out of range fails early
- **WHEN** a client requests an offset outside [-12, 14]
- **THEN** the bridge validates the path parameter, responds with HTTP 400, and does not reach Pixoo so the device never receives an invalid timezone string

#### Scenario: Non-numeric offset is rejected
- **WHEN** a client sends `POST /manage/time/offset/abc`
- **THEN** the bridge rejects the request with HTTP 400 and never issues a Pixoo command because the path parameter cannot be parsed as an integer

### Requirement: Manage device UTC clock command
The bridge SHALL expose `POST /manage/time` so callers can trigger the Pixoo device’s UTC clock update without providing a body. The handler SHALL compute the current UTC instant using the system clock, convert it to seconds since the epoch, and then call `/post` with `Command: "Device/SetUTC"` and that computed `Utc` value, mapping Pixoo failures to HTTP 503 after retries are exhausted.

#### Scenario: UTC time is forwarded
- **WHEN** a client calls `POST /manage/time` (no body)
- **THEN** the bridge reads the current UTC timestamp, sends `{ "Command": "Device/SetUTC", "Utc": <current seconds> }` to `/post`, waits for retries, and returns HTTP 200 once Pixoo accepts the update

#### Scenario: System clock cannot be read
- **WHEN** the bridge fails to read the system clock for UTC calculation
- **THEN** the bridge responds with HTTP 500 and does not issue a Pixoo command so clients know the time update could not be attempted
