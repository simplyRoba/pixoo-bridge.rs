# api/health Capability

## Purpose
Describe the bridge’s health endpoint behavior so container operators know how to interpret probes and the optional Pixoo forwarding signal.

## Requirements

### Requirement: Bridge health endpoint
The bridge SHALL expose an HTTP GET `/health` endpoint for container health checks.

#### Scenario: Health endpoint responds
- **WHEN** a client sends `GET /health`
- **THEN** the bridge responds with HTTP 200 and a JSON body containing `{ "status": "ok" }`

### Requirement: Forwarded health behavior
When health forwarding is enabled, the bridge SHALL issue a Pixoo GET `/get` request and treat any non-200 status as unhealthy.

#### Scenario: Pixoo health success
- **WHEN** the Pixoo device responds to `GET /get` with HTTP 200
- **THEN** the bridge responds to `GET /health` with HTTP 200 and `{ "status": "ok" }`

#### Scenario: Pixoo health failure
- **WHEN** the Pixoo device responds to `GET /get` with a non-200 status
- **THEN** the bridge responds to `GET /health` with HTTP 503
