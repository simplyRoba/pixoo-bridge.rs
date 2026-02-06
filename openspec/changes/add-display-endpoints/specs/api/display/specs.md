# api/display Capability

## Purpose
Expose HTTP endpoints under `/manage/manage/display/*` to control and manage the Pixoo display device settings. These endpoints provide a user-friendly interface for toggling the display on/off, adjusting brightness, rotation, mirroring, highlight mode, and white balance.

## Requirements

### Requirement: Toggle display on/off
The bridge SHALL expose `POST /manage/display/on/{action}` to control the display power state. The endpoint SHALL accept `on` or `off` as the `{action}` path parameter.
- If `{action}` is `on`, the bridge SHALL send `POST /post` with `Command: "Channel/OnOffScreen"` and `OnOff: 1`.
- If `{action}` is `off`, the bridge SHALL send `POST /post` with `Command: "Channel/OnOffScreen"` and `OnOff: 0`.

#### Scenario: Turn display on
- **WHEN** a client sends `POST /manage/display/on/on`
- **THEN** the bridge posts `{ "Command": "Channel/OnOffScreen", "OnOff": 1 }` to Pixoo and returns HTTP 200 with `{ "error_code": 0 }`

#### Scenario: Turn display off
- **WHEN** a client sends `POST /manage/display/on/off`
- **THEN** the bridge posts `{ "Command": "Channel/OnOffScreen", "OnOff": 0 }` to Pixoo and returns HTTP 200 with `{ "error_code": 0 }`

#### Scenario: Invalid action
- **WHEN** a client sends `POST /manage/display/on/invalid`
- **THEN** the bridge returns HTTP 400 and does NOT send a command to Pixoo

### Requirement: Set display brightness
The bridge SHALL expose `POST /manage/display/brightness/{value}` to adjust the display brightness. The endpoint SHALL accept an integer `{value}` between 0 and 100.

#### Scenario: Valid brightness value
- **WHEN** a client sends `POST /manage/display/brightness/75`
- **THEN** the bridge posts `{ "Command": "Channel/SetBrightness", "Brightness": 75 }` to Pixoo and returns HTTP 200 with `{ "error_code": 0 }`

#### Scenario: Brightness out of range
- **WHEN** a client sends `POST /manage/display/brightness/150`
- **THEN** the bridge returns HTTP 400 and does NOT send a command to Pixoo

#### Scenario: Non-numeric brightness value
- **WHEN** a client sends `POST /manage/display/brightness/abc`
- **THEN** the bridge returns HTTP 400 and does NOT send a command to Pixoo

### Requirement: Set display rotation
The bridge SHALL expose `POST /manage/display/rotation/{angle}` to rotate the display. The endpoint SHALL accept an integer `{angle}` representing the rotation in degrees (0, 90, 180, or 270).

#### Scenario: Valid rotation angle
- **WHEN** a client sends `POST /manage/display/rotation/90`
- **THEN** the bridge posts `{ "Command": "Device/SetScreenRotationAngle", "Mode": 1 }` to Pixoo and returns HTTP 200 with `{ "error_code": 0 }`

#### Scenario: Invalid rotation angle
- **WHEN** a client sends `POST /manage/display/rotation/45`
- **THEN** the bridge returns HTTP 400 and does NOT send a command to Pixoo

### Requirement: Toggle mirror mode
The bridge SHALL expose `POST /manage/display/mirror/{action}` to enable or disable mirror mode. The endpoint SHALL accept `on` or `off` as the `{action}` path parameter.
- If `{action}` is `on`, the bridge SHALL send `POST /post` with `Command: "Device/SetMirrorMode"` and `Mode: 1`.
- If `{action}` is `off`, the bridge SHALL send `POST /post` with `Command: "Device/SetMirrorMode"` and `Mode: 0`.

#### Scenario: Enable mirror mode
- **WHEN** a client sends `POST /manage/display/mirror/on`
- **THEN** the bridge posts `{ "Command": "Device/SetMirrorMode", "Mode": 1 }` to Pixoo and returns HTTP 200 with `{ "error_code": 0 }`

#### Scenario: Disable mirror mode
- **WHEN** a client sends `POST /manage/display/mirror/off`
- **THEN** the bridge posts `{ "Command": "Device/SetMirrorMode", "Mode": 0 }` to Pixoo and returns HTTP 200 with `{ "error_code": 0 }`

#### Scenario: Invalid mirror action
- **WHEN** a client sends `POST /manage/display/mirror/invalid`
- **THEN** the bridge returns HTTP 400 and does NOT send a command to Pixoo

### Requirement: Toggle highlight mode
The bridge SHALL expose `POST /manage/display/highlight/{action}` to enable or disable highlight mode. The endpoint SHALL accept `on` or `off` as the `{action}` path parameter.
- If `{action}` is `on`, the bridge SHALL send `POST /post` with `Command: "Device/SetHighLightMode"` and `Mode: 1`.
- If `{action}` is `off`, the bridge SHALL send `POST /post` with `Command: "Device/SetHighLightMode"` and `Mode: 0`.

#### Scenario: Enable highlight mode
- **WHEN** a client sends `POST /manage/display/highlight/on`
- **THEN** the bridge posts `{ "Command": "Device/SetHighLightMode", "Mode": 1 }` to Pixoo and returns HTTP 200 with `{ "error_code": 0 }`

#### Scenario: Disable highlight mode
- **WHEN** a client sends `POST /manage/display/highlight/off`
- **THEN** the bridge posts `{ "Command": "Device/SetHighLightMode", "Mode": 0 }` to Pixoo and returns HTTP 200 with `{ "error_code": 0 }`

#### Scenario: Invalid highlight action
- **WHEN** a client sends `POST /manage/display/highlight/invalid`
- **THEN** the bridge returns HTTP 400 and does NOT send a command to Pixoo

### Requirement: Set white balance
The bridge SHALL expose `POST /manage/display/white-balance` to adjust the display's white balance. The endpoint SHALL accept a JSON body with `red`, `green`, and `blue` values, each an integer between 0 and 100.

#### Scenario: Valid white balance values
- **WHEN** a client posts `{ "red": 90, "green": 100, "blue": 110 }` to `/manage/display/white-balance`
- **THEN** the bridge posts `{ "Command": "Device/SetWhiteBalance", "RValue": 90, "GValue": 100, "BValue": 110 }` to Pixoo and returns HTTP 200 with `{ "error_code": 0 }`

#### Scenario: White balance values out of range
- **WHEN** a client posts `{ "red": 150, "green": 100, "blue": 100 }` to `/manage/display/white-balance`
- **THEN** the bridge returns HTTP 400 and does NOT send a command to Pixoo

#### Scenario: Missing white balance values
- **WHEN** a client posts `{ "red": 100 }` to `/manage/display/white-balance`
- **THEN** the bridge returns HTTP 400 and does NOT send a command to Pixoo
