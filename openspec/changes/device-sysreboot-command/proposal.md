## Why

Automations and scripts using the Pixoo bridge currently cannot trigger a device reboot, which leaves customers manually restarting the matrix when it misbehaves. Exposing the Pixoo `Device/SysReboot` command through the bridge’s HTTP API gives operators a reliable, documented way to restart the device remotely.

## What Changes

- Introduce a capability that sends the Pixoo `Device/SysReboot` command and handle any framing/backoff constraints the bridge already applies to other system commands.
- Add an HTTP endpoint at `/reboot` that maps to the new capability, keeping the request/response shapes empty so the command remains a simple trigger.
- Update the bridge documentation and command routing layer to describe and wire the new endpoint.

## Capabilities

### New Capabilities
- `api/system`: send `Device/SysReboot` over Pixoo UDP and expose an empty `/reboot` REST route for users to trigger it.

### Modified Capabilities
- `api/health`: reorganize the capability under `api/system` so its monitoring endpoint shares the same domain as `/reboot` and benefits from the same routing abstractions.

## Impact

- `pixoo_bridge::pixoo::Command` definitions and the command router need a new opcode/handler for `Device/SysReboot`.
- `pixoo_bridge::http::routes` must gain a `/reboot` handler that matches current auth/retry guards and delegates to the new command capability.
- Documentation (README/CHANGELOG) should mention the new endpoint so integrators can discover it.
