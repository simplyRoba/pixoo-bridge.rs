# core/configuration Capability

## Requirements

### Requirement: Configurable HTTP listener port
The bridge SHALL honor `PIXOO_BRIDGE_PORT` when provided, parsing it as a `u16` and ensuring it falls within `1024..=65535`. When the value is missing or invalid the service SHALL default to `4000` without panicking, and runtime logs SHALL mention the port that was bound so operators can verify the listener.

#### Scenario: Default listener port
- **WHEN** `PIXOO_BRIDGE_PORT` is undefined in the environment
- **THEN** the bridge binds Axum to port `4000` and emits an info-level log entry referencing the bound port and Pixoo base URL (if configured)

#### Scenario: Valid custom port
- **WHEN** `PIXOO_BRIDGE_PORT` is set to a valid user-space port (for example `5005`)
- **THEN** the bridge binds to that port and logs the configured value at startup so deployment tooling can confirm the mapping

#### Scenario: Invalid port value
- **WHEN** `PIXOO_BRIDGE_PORT` is set to a non-numeric or out-of-range value
- **THEN** the bridge logs a warning naming the invalid provisioned value and falls back to port `4000` to keep the HTTP endpoint reachable

### Requirement: Log level configurable via environment variable
The bridge SHALL honor `PIXOO_BRIDGE_LOG_LEVEL` by mapping it to the logging framework’s level filter while defaulting to `info` so operators can increase or decrease verbosity without rebuilding the container.

#### Scenario: Environment overrides level
- **WHEN** `PIXOO_BRIDGE_LOG_LEVEL=debug` is set
- **THEN** the bridge initializes tracing with the `debug` filter so debug statements become visible in the container logs

#### Scenario: Invalid value falls back to info
- **WHEN** `PIXOO_BRIDGE_LOG_LEVEL` contains an unsupported value
- **THEN** the bridge logs a warning about the invalid setting and continues with `info` as the active level
