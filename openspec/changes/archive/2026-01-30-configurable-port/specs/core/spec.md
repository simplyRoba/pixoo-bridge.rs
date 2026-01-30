## ADDED Requirements

### Requirement: Configurable HTTP listener port
The system SHALL honor `PIXOO_BRIDGE_PORT` when provided, parsing it as a `u16` and ensuring it falls within `1024..=65535`. When the value is missing or invalid the service SHALL default to `4000` without panicking, and runtime logs SHALL mention the port that was bound so operators can verify the listener.

#### Scenario: Default listener port
- **WHEN** `PIXOO_BRIDGE_PORT` is undefined in the environment
- **THEN** the bridge binds Axum to port `4000` and emits an info-level log entry referencing the bound port and Pixoo base URL (if configured)

#### Scenario: Valid custom port
- **WHEN** `PIXOO_BRIDGE_PORT` is set to a valid user-space port (for example `5005`)
- **THEN** the bridge binds to that port and logs the configured value at startup so deployment tooling can confirm the mapping

#### Scenario: Invalid port value
- **WHEN** `PIXOO_BRIDGE_PORT` is set to a non-numeric or out-of-range value
- **THEN** the bridge logs a warning naming the invalid provisioned value and falls back to port `4000` to keep the HTTP endpoint reachable
