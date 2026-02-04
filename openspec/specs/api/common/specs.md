# api/common Capability

## Purpose
Ensure shared validation models and enums protect every `/tools` extractor so normalized data reaches Pixoo while invalid requests are rejected consistently.

## ADDED Requirements

### Requirement: Request payloads are validated consistently
The bridge SHALL require every HTTP handler to accept requests through dedicated structs that derive `Deserialize` and `Validate`, document numeric/text constraints through validator attributes, and reject malformed payloads before any Pixoo interaction so downstream logic only sees normalized data. Example: `/tools/scoreboard` or any future tool payload must pass the shared validation layer.

#### Scenario: Valid payload is forwarded
- **WHEN** a handler receives values inside the documented ranges with required fields present
- **THEN** the shared request model deserializes/validates successfully and the handler forwards the normalized payload to Pixoo while keeping the JSON response unchanged.

#### Scenario: Invalid payload is rejected before Pixoo
- **WHEN** a handler receives JSON with out-of-range numbers, missing required fields, or invalid text
- **THEN** validation fails, the handler returns a 400-series error, and Pixoo is never invoked with the malformed data.

### Requirement: Tool action routes accept only enum variants
The bridge SHALL deserialize action path parameters for any tool-specific route through enums that derive `Deserialize` with `rename_all = "lowercase"`, ensuring unsupported values are rejected before command translation. Example: `/tools/stopwatch/{action}` or `/tools/soundmeter/{action}` use the shared action enums.

#### Scenario: Supported action enum routes succeed
- **WHEN** a client sends a lowercase action value that matches a supported enum variant
- **THEN** the enum parameter deserializes, the handler executes the mapped Pixoo command, and the JSON response remains unchanged.

#### Scenario: Unsupported action returns validation error
- **WHEN** a client calls a tool action route with an unsupported action value
- **THEN** deserialization fails, the handler responds with a 400-series error, and Pixoo never receives a command for the unknown action.

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
