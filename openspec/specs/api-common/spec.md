# api/common Capability

## Purpose
Ensure shared validation models and enums protect every `/tools` extractor so normalized data reaches Pixoo while invalid requests are rejected consistently.

## Requirements

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
Every error response from the bridge SHALL use one canonical error envelope whose root object always contains exactly three fields — `error_status` (the HTTP status as an integer), `error_kind` (a string discriminator), and `message` (human-readable text). All case-specific data SHALL be placed inside a single nested `details` object; no case-specific field may appear at the root, and `details` SHALL be omitted entirely when there is no extra data. This applies to validation failures, not-found responses, payload-too-large responses, and Pixoo device/server failures alike, so downstream systems can parse every error with a single shape.

The `error_kind` discriminator SHALL be one of: `validation`, `not-found`, `payload-too-large`, `unreachable`, `timeout`, `device-error`, `remote-fetch`, `internal`.

#### Scenario: Device failure returns the canonical envelope
- **WHEN** a Pixoo-facing endpoint encounters a mapped device error (unreachable, timeout, or device error)
- **THEN** it responds with the mapped HTTP status (502/504/503) and a body whose root is `{ "error_status": <int>, "error_kind": "<unreachable|timeout|device-error>", "message": "<text>" }`
- **AND** when the device provided an error code, that code appears only as `details.error_code`, never at the root

#### Scenario: Validation failure returns the canonical envelope
- **WHEN** a handler rejects a request due to payload or path validation
- **THEN** it responds with status 400 and a body whose root is `{ "error_status": 400, "error_kind": "validation", "message": "<text>" }`
- **AND** the per-field or per-action information appears under `details` (e.g. `details: { "red": ["range"] }`)

#### Scenario: Payload-too-large failure returns the canonical envelope
- **WHEN** a handler rejects an upload or remote payload that exceeds the configured size limit
- **THEN** it responds with status 413 and a body whose root is `{ "error_status": 413, "error_kind": "payload-too-large", "message": "<text>" }`
- **AND** the size information appears as `details: { "limit": <int>, "actual": <int> }`, not at the root

#### Scenario: Not-found and extra-less failures omit details
- **WHEN** a request hits an unknown route, or any error with no kind-specific data occurs (e.g. a timeout)
- **THEN** the response body is exactly `{ "error_status": <int>, "error_kind": "<kind>", "message": "<text>" }` with no `details` key present

#### Scenario: Legacy `error` field is gone
- **WHEN** any error response is produced
- **THEN** the body does NOT contain a root `error` string field, and case-specific keys such as `limit`, `actual`, or `error_code` do NOT appear at the root
