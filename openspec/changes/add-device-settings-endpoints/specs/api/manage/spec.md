## ADDED Requirements

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
