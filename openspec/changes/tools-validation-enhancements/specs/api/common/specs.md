## ADDED Requirements

### Requirement: Tool action routes accept only enum variants
The bridge SHALL deserialize action path parameters for any tool-specific route through enums that derive `Deserialize` with `rename_all = "lowercase"`, ensuring unsupported values are rejected before command translation. Example: `/tools/stopwatch/{action}` or `/tools/soundmeter/{action}` use the shared action enums.

#### Scenario: Supported action enum routes succeed
- **WHEN** a client sends a lowercase action value that matches a supported enum variant
- **THEN** the enum parameter deserializes, the handler executes the mapped Pixoo command, and the JSON response remains unchanged.

#### Scenario: Unsupported action returns validation error
- **WHEN** a client calls a tool action route with an unsupported action value
- **THEN** deserialization fails, the handler responds with a 400-series error, and Pixoo never receives a command for the unknown action.
