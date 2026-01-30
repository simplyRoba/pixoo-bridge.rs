# api/health Capability

# api/health Capability

## ADDED Requirements

### Requirement: Bridge health endpoint
The bridge SHALL expose an HTTP GET `/health` endpoint for container health checks.

#### Scenario: Health endpoint responds
- **WHEN** a client sends `GET /health`
- **THEN** the bridge responds with HTTP 200 and a JSON body containing `{ "status": "ok" }`


### Requirement: Health forwarding toggle
The bridge SHALL read `PIXOO_BRIDGE_HEALTH_FORWARD` to control whether the health endpoint cascades to the Pixoo device, defaulting to `true` when unset.

#### Scenario: Forwarding enabled by default
- **WHEN** `PIXOO_BRIDGE_HEALTH_FORWARD` is unset
- **THEN** the bridge performs a Pixoo health check as part of `GET /health`

#### Scenario: Forwarding disabled
- **WHEN** `PIXOO_BRIDGE_HEALTH_FORWARD` is set to `false`
- **THEN** the bridge responds with HTTP 200 without contacting the Pixoo device

### Requirement: Forwarded health behavior
When health forwarding is enabled, the bridge SHALL issue a Pixoo GET `/get` request and treat any non-200 status as unhealthy.

#### Scenario: Pixoo health success
- **WHEN** the Pixoo device responds to `GET /get` with HTTP 200
- **THEN** the bridge responds to `GET /health` with HTTP 200 and `{ "status": "ok" }`

#### Scenario: Pixoo health failure
- **WHEN** the Pixoo device responds to `GET /get` with a non-200 status
- **THEN** the bridge responds to `GET /health` with HTTP 503
