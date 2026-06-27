## MODIFIED Requirements

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
