# core/configuration Capability

## MODIFIED Requirements

### Requirement: Health forwarding toggle
The bridge SHALL read `PIXOO_BRIDGE_HEALTH_FORWARD` to control whether the health endpoint cascades to the Pixoo device, defaulting to `true` when unset.

#### Scenario: Forwarding enabled by default
- **WHEN** `PIXOO_BRIDGE_HEALTH_FORWARD` is unset
- **THEN** the bridge performs a Pixoo health check as part of `GET /health`

#### Scenario: Forwarding disabled
- **WHEN** `PIXOO_BRIDGE_HEALTH_FORWARD` is set to `false`
- **THEN** the bridge responds with HTTP 200 without contacting the Pixoo device
