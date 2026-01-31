## Why

Pixoo devices expose an undocumented `/get` endpoint that can be used to verify device health, but the bridge currently cannot surface that signal for container healthchecks. We need a consistent bridge-level healthcheck that can optionally cascade to the device to improve operational reliability.

## What Changes

- Add a bridge healthcheck endpoint that can be used by Docker health probes.
- Teach the Pixoo client to call the device `/get` endpoint when health forwarding is enabled.
- Add `PIXOO_BRIDGE_HEALTH_FORWARD` to control cascading (default: true).

## Capabilities

### New Capabilities
- `health`: Bridge HTTP healthcheck behavior for downstream clients.

### Modified Capabilities
- `pixoo-client`: Pixoo HTTP client requirements expand to include `/get` healthcheck calls.

## Impact

- `src/pixoo` client adds a GET request path for health checks.
- Bridge HTTP routing adds a healthcheck endpoint used by Docker.
- Configuration/env handling adds `PIXOO_BRIDGE_HEALTH_FORWARD` with default true.
- Container healthcheck guidance likely updated to target the bridge endpoint.
