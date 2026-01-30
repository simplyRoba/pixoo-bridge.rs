# core/startup-logging Capability

## ADDED Requirements

### Requirement: Startup logging records runtime configuration
The bridge SHALL emit an info-level log once at startup that lists the resolved health forwarding flag, the sanitized Pixoo base URL (scheme and host only), and the listener address so operators know what settings the container began with.

#### Scenario: Container starts with health forwarding enabled
- **WHEN** the service finishes building `AppState` or equivalent and before it accepts HTTP traffic
- **THEN** it logs an info entry containing `health_forward=true`, the sanitized base URL, and the listener address

### Requirement: Log level configurable via environment variable
The bridge SHALL honor `PIXOO_BRIDGE_LOG_LEVEL` by mapping it to the logging framework’s level filter while defaulting to `info` so operators can increase or decrease verbosity without rebuilding the container.

#### Scenario: Environment overrides level
- **WHEN** `PIXOO_BRIDGE_LOG_LEVEL=debug` is set
- **THEN** the bridge initializes tracing with the `debug` filter so debug statements become visible in the container logs

#### Scenario: Invalid value falls back to info
- **WHEN** `PIXOO_BRIDGE_LOG_LEVEL` contains an unsupported value
- **THEN** the bridge logs a warning about the invalid setting and continues with `info` as the active level
