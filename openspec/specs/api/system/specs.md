# api/system Capability

## Purpose
Bring the `/health` and `/reboot` endpoints together under one domain so operators see system-level maintenance hooks in a single capability while reusing the shared routing and middleware pipeline built for Pixoo’s unreliable UDP API.

## Requirements

### Requirement: Bridge health endpoint
The bridge SHALL expose an HTTP GET `/health` endpoint for container health checks through the `api/system` routing surface so that system probes and reboot operations share common diagnostics and auth middleware.

#### Scenario: Health endpoint responds
- **WHEN** a client sends `GET /health`
- **THEN** the bridge responds with HTTP 200 and a JSON body containing `{ "status": "ok" }`

### Requirement: Forwarded health behavior
When health forwarding is enabled, the bridge SHALL issue a Pixoo GET `/get` request and treat any non-200 status as unhealthy, still within the `api/system` domain.

#### Scenario: Pixoo health success
- **WHEN** the Pixoo device responds to `GET /get` with HTTP 200
- **THEN** the bridge responds to `GET /health` with HTTP 200 and `{ "status": "ok" }`

#### Scenario: Pixoo health failure
- **WHEN** the Pixoo device responds to `GET /get` with a non-200 status
- **THEN** the bridge responds to `GET /health` with HTTP 503

### Requirement: Device system reboot command
The bridge SHALL expose an HTTP POST `/reboot` endpoint with no request body so clients can trigger the Pixoo `Device/SysReboot` command without sending additional data.

#### Scenario: Reboot command accepted
- **WHEN** a client sends `POST /reboot` with an empty body
- **THEN** the bridge issues `Device/SysReboot` to the Pixoo device, waits for the existing retry/backoff helper, and responds with HTTP 200 OK once the command is accepted

#### Scenario: Reboot command fails
- **WHEN** Pixoo does not acknowledge `Device/SysReboot` after the configured retries or returns an error
- **THEN** the bridge responds to `POST /reboot` with HTTP 503 and a short error message so operators know to retry
