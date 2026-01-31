# core/pixoo-client Capability

## Purpose
Clarify how Pixoo commands and health checks should be serialized, dispatched, and validated so downstream code knows how to interact with the device consistently.

## Requirements

### Requirement: Command payload construction
The client SHALL construct a JSON request body that includes the `Command` field derived from a command enum plus all provided argument fields flattened into the same JSON object.

#### Scenario: Command with additional fields
- **WHEN** the caller issues a `Tools/SetTimer` command with `Minute`, `Second`, and `Status` arguments
- **THEN** the client sends a JSON object containing `Command`, `Minute`, `Second`, and `Status` in the request body

### Requirement: HTTP request shape
The client SHALL send Pixoo commands via HTTP POST to a configured device IP and set the request `Content-Type` to `application/json`. The client SHALL send Pixoo health checks via HTTP GET to the device `/get` endpoint without a request body.

#### Scenario: Post command to device
- **WHEN** the caller sends any Pixoo command
- **THEN** the client issues an HTTP POST to the configured device endpoint with `Content-Type: application/json`

#### Scenario: Get health from device
- **WHEN** the caller requests a Pixoo health check
- **THEN** the client issues an HTTP GET to the device `/get` endpoint

### Requirement: Response parsing with incorrect content type
The client SHALL parse the response body as JSON regardless of the response `Content-Type` header value.

#### Scenario: Response labeled text/html
- **WHEN** the device responds with `Content-Type: text/html` and a JSON body
- **THEN** the client parses the body as JSON and makes the fields available to the caller

### Requirement: Error code validation
The client SHALL read `error_code` from every response and treat any non-zero value as a failure.

#### Scenario: Device returns error
- **WHEN** the device responds with `error_code` set to a non-zero value
- **THEN** the client returns an error that includes the `error_code`

### Requirement: Successful response shaping
On successful responses (`error_code` equals zero), the client SHALL return the remaining response fields without `error_code`.

#### Scenario: Get command response fields
- **WHEN** the device responds with `error_code: 0` plus additional fields such as `Brightness` and `RotationFlag`
- **THEN** the client returns a response map containing the additional fields and omits `error_code`

### Requirement: Container healthcheck
The Docker image SHALL define a container healthcheck that calls `GET /health` on the bridge.

#### Scenario: Container healthcheck configured
- **WHEN** the Docker image is built
- **THEN** the container healthcheck invokes the bridge `/health` endpoint
