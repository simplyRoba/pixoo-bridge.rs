## ADDED Requirements

### Requirement: HTTP status classification for Pixoo client failures
The bridge SHALL classify Pixoo client error outcomes and respond with corresponding HTTP statuses so callers can distinguish unreachable devices (502), device health failures (503), and client timeouts (504) while keeping 503 as the conservative default for uncategorized errors.

#### Scenario: Device unreachable after retries
- **WHEN** the Pixoo client exhausts all retry/backoff attempts without opening a connection
- **THEN** the HTTP handler returns status 502, and the response body indicates the device could not be reached.

#### Scenario: Device responds with a failure payload
- **WHEN** the Pixoo device replies with an `error_code` or other failure indicator after a successful connection
- **THEN** the handler returns status 503 along with the device-provided error details so operators know the device itself reported a problem.

#### Scenario: Pixoo request times out mid-flight
- **WHEN** the Pixoo client hits its timeout threshold before receiving a response
- **THEN** the handler returns status 504 and mentions the timeout in the error body, signaling the upstream call never completed.

### Requirement: Structured HTTP error payloads for all mapped statuses
Every Pixoo-exposed handler SHALL use the shared error-mapping helper to build an error response containing `error_status`, `message`, and either `error_code` or `error_kind` so that downstream systems can parse the mapping without relying on unstructured logs.

#### Scenario: Handler returns structured payload
- **WHEN** any Pixoo-facing endpoint encounters an error classified by the mapping
- **THEN** it responds with the mapped HTTP status and a JSON body resembling `{ "error_status": <int>, "message": "<human-readable text>", "error_kind": "<unreachable|timeout|device-error>", "error_code": <optional Pixoo code> }`.
