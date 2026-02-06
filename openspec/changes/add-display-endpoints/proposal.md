## Why

The Pixoo bridge currently lacks a dedicated API surface for controlling display-specific settings like power state, brightness, rotation, mirroring, overclocking, and white balance. These settings are currently managed through the `/manage` endpoints or require direct Pixoo API knowledge. This change introduces a dedicated `/display/*` endpoint group to provide a clear, user-friendly interface for display management, improving usability for automation systems and end users.

## What Changes

- **ADDED**: New `/manage/display/on/{action}` endpoint to toggle display power state (on/off)
- **ADDED**: New `/manage/display/brightness/{value}` endpoint to set display brightness (0-100)
- **ADDED**: New `/manage/display/rotation/{angle}` endpoint to rotate the display (0, 90, 180, 270 degrees)
- **ADDED**: New `/manage/display/mirror/{action}` endpoint to enable/disable mirror mode
- **ADDED**: New `/manage/display/brightness/overclock/{action}` endpoint to control overclock mode (on/off)
- **ADDED**: New `/manage/display/white-balance` endpoint to adjust RGB white balance values

## Capabilities

### New Capabilities
- `api/manage`: Extended manage endpoints with display control functionality

### Modified Capabilities
- None

## Impact

- **API**: New endpoints under `/manage/display/*` path prefix
- **Code**: New handler functions in the API module
- **Dependencies**: Uses existing Pixoo client and retry/backoff helpers
- **Systems**: No external system dependencies; works with existing Pixoo device communication
