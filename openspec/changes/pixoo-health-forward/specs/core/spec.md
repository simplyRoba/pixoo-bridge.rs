## MODIFIED Requirements

### Requirement: HTTP request shape
The client SHALL send Pixoo commands via HTTP POST to a configured device IP and set the request `Content-Type` to `application/json`. The client SHALL send Pixoo health checks via HTTP GET to the device `/get` endpoint without a request body.

#### Scenario: Post command to device
- **WHEN** the caller sends any Pixoo command
- **THEN** the client issues an HTTP POST to the configured device endpoint with `Content-Type: application/json`

#### Scenario: Get health from device
- **WHEN** the caller requests a Pixoo health check
- **THEN** the client issues an HTTP GET to the device `/get` endpoint
