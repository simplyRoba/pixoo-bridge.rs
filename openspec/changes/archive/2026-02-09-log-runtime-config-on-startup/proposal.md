## Why

The startup log already surfaces the Pixoo base URL, listener address, and version, but operators still have to dig into configuration for animation tuning and payload-size guards. Recording the resolved animation speed factor and maximum image size when the bridge begins running makes these runtime controls visible in the logs and helps diagnose behavior differences between deployments.

## What Changes

- Capture `animation_speed_factor` and `max_image_size` in the structured startup log entry so operators see the resolved tuning values alongside the existing configuration metadata.
- Update the logging capability’s startup requirement to document the additional fields and keep the spec aligned with the emitted log.
- No new APIs or dependencies are required; this change only touches configuration loading/logging and documentation.

## Capabilities

### New Capabilities
- None

### Modified Capabilities
- `core/logging`: the startup logging requirement is extended to include the resolved animation speed factor and maximum image size alongside the existing configuration details.

## Impact

- `src/main.rs` (startup logging) and any helpers that format the log entry.
- `openspec/specs/core/logging/specs.md` to keep the documented requirements in sync with the emitted log.
- Operator observability; no API surface changes.
