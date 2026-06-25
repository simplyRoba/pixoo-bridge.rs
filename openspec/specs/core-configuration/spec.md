# core/configuration Capability

## Purpose
Document how configuration knobs (listener port, log level, health forwarding, Pixoo base URL) shape the bridge so operators can understand how runtime settings affect behavior.

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

### Requirement: Health forwarding toggle
The bridge SHALL read `PIXOO_BRIDGE_HEALTH_FORWARD` to control whether the health endpoint cascades to the Pixoo device, defaulting to `true` when unset.

#### Scenario: Forwarding enabled by default
- **WHEN** `PIXOO_BRIDGE_HEALTH_FORWARD` is unset
- **THEN** the bridge performs a Pixoo health check as part of `GET /health`

#### Scenario: Forwarding disabled
- **WHEN** `PIXOO_BRIDGE_HEALTH_FORWARD` is set to `false`
- **THEN** the bridge responds with HTTP 200 without contacting the Pixoo device

### Requirement: Log level configurable via environment variable
The bridge SHALL honor `PIXOO_BRIDGE_LOG_LEVEL` by mapping it to the logging framework’s level filter while defaulting to `info` so operators can increase or decrease verbosity without rebuilding the container.

#### Scenario: Environment overrides level
- **WHEN** `PIXOO_BRIDGE_LOG_LEVEL=debug` is set
- **THEN** the bridge initializes tracing with the `debug` filter so debug statements become visible in the container logs

#### Scenario: Invalid value falls back to info
- **WHEN** `PIXOO_BRIDGE_LOG_LEVEL` contains an unsupported value
- **THEN** the bridge logs a warning about the invalid setting and continues with `info` as the active level

### Requirement: Startup configuration passed to Pixoo client
The bridge SHALL load Pixoo client configuration during startup and construct the Pixoo client using those explicit values.

#### Scenario: Startup configuration applied to client
- **GIVEN** `PIXOO_BASE_URL` and timeout settings are configured
- **WHEN** the bridge starts
- **THEN** the Pixoo client is constructed with the configured values without requiring additional environment reads during client construction

### Requirement: Pixoo base URL configuration
The bridge SHALL require `PIXOO_BASE_URL` to be configured at startup and include the configured URL in the startup info log so deployment tooling can confirm where the bridge is directing commands.

#### Scenario: Base URL supplied
- **WHEN** `PIXOO_BASE_URL` is set (e.g., `http://10.0.0.5`)
- **THEN** the bridge uses that host to reach the Pixoo device and logs a `pixoo_base_url` field containing the configured URL

#### Scenario: Base URL omitted
- **WHEN** `PIXOO_BASE_URL` is unset
- **THEN** the process exits with a non-zero status and logs a clear configuration error

### Requirement: Animation speed factor configuration
The bridge SHALL read `PIXOO_ANIMATION_SPEED_FACTOR` as a floating-point multiplier applied to per-frame delays read from animated files. The default SHALL be `1.4`. Invalid or non-positive values SHALL fall back to the default with a warning logged. The parsed value SHALL be stored in `AppConfig` and threaded into `AppState`.

#### Scenario: Default animation speed factor
- **WHEN** `PIXOO_ANIMATION_SPEED_FACTOR` is undefined in the environment
- **THEN** the bridge uses `1.4` as the animation speed factor

#### Scenario: Valid custom speed factor
- **WHEN** `PIXOO_ANIMATION_SPEED_FACTOR` is set to `2.0`
- **THEN** the bridge uses `2.0` as the animation speed factor and all animated frame delays are multiplied by `2.0`

#### Scenario: Invalid speed factor falls back to default
- **WHEN** `PIXOO_ANIMATION_SPEED_FACTOR` is set to `abc`
- **THEN** the bridge logs a warning naming the invalid value and falls back to `1.4`

#### Scenario: Non-positive speed factor falls back to default
- **WHEN** `PIXOO_ANIMATION_SPEED_FACTOR` is set to `0` or `-1.0`
- **THEN** the bridge logs a warning and falls back to `1.4`

### Requirement: Maximum image size configuration
The bridge SHALL read `PIXOO_BRIDGE_MAX_IMAGE_SIZE` as a human-readable byte size limiting uploaded image files. The value SHALL accept formats like `5MB`, `128KB`, `1024B` (case-insensitive, with or without the trailing `B` — e.g. `5M` and `5MB` are equivalent). The bridge SHALL use binary units (1 KB = 1024 bytes). The default SHALL be `5MB` (5,242,880 bytes). Invalid values SHALL fall back to the default with a warning logged. The parsed value SHALL be stored in `AppConfig` and threaded into `AppState`.

#### Scenario: Default max image size
- **WHEN** `PIXOO_BRIDGE_MAX_IMAGE_SIZE` is undefined in the environment
- **THEN** the bridge uses 5,242,880 bytes (5 MB) as the maximum image size

#### Scenario: Valid custom size in megabytes
- **WHEN** `PIXOO_BRIDGE_MAX_IMAGE_SIZE` is set to `10MB`
- **THEN** the bridge uses 10,485,760 bytes as the maximum image size

#### Scenario: Valid custom size in kilobytes without B suffix
- **WHEN** `PIXOO_BRIDGE_MAX_IMAGE_SIZE` is set to `128K`
- **THEN** the bridge uses 131,072 bytes as the maximum image size

#### Scenario: Invalid size value falls back to default
- **WHEN** `PIXOO_BRIDGE_MAX_IMAGE_SIZE` is set to `lots`
- **THEN** the bridge logs a warning naming the invalid value and falls back to 5,242,880 bytes
