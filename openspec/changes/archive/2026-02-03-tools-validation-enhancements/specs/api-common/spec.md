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
