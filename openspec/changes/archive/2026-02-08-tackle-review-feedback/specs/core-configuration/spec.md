## ADDED Requirements

### Requirement: Startup configuration passed to Pixoo client
The bridge SHALL load Pixoo client configuration during startup and construct the Pixoo client using those explicit values.

#### Scenario: Startup configuration applied to client
- **GIVEN** `PIXOO_BASE_URL` and timeout settings are configured
- **WHEN** the bridge starts
- **THEN** the Pixoo client is constructed with the configured values without requiring additional environment reads during client construction

## MODIFIED Requirements

### Requirement: Pixoo base URL configuration
The bridge SHALL require `PIXOO_BASE_URL` to be configured at startup and include the configured URL in the startup info log so deployment tooling can confirm where the bridge is directing commands.

#### Scenario: Base URL supplied
- **GIVEN** `PIXOO_BASE_URL` is set (for example `http://10.0.0.5`)
- **WHEN** the bridge starts
- **THEN** the bridge uses that host to reach the Pixoo device and logs a `pixoo_base_url` field containing the configured URL

#### Scenario: Base URL omitted
- **GIVEN** `PIXOO_BASE_URL` is unset
- **WHEN** the bridge starts
- **THEN** the process exits with a non-zero status and logs a clear configuration error
