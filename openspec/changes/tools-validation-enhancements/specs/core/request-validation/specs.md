## ADDED Requirements

### Requirement: Request payloads are validated consistently
The bridge SHALL require every HTTP handler to accept requests through dedicated structs that derive `Deserialize` and `Validate`, document numeric/text constraints through validator attributes, and reject malformed payloads before any Pixoo interaction so downstream logic only sees normalized data. Example: `/tools/scoreboard` or any future tool payload must pass the shared validation layer.

#### Scenario: Valid payload is forwarded
- **WHEN** a handler receives values inside the documented ranges with required fields present
- **THEN** the shared request model deserializes/validates successfully and the handler forwards the normalized payload to Pixoo while keeping the JSON response unchanged.

#### Scenario: Invalid payload is rejected before Pixoo
- **WHEN** a handler receives JSON with out-of-range numbers, missing required fields, or invalid text
- **THEN** validation fails, the handler returns a 400-series error, and Pixoo is never invoked with the malformed data.
