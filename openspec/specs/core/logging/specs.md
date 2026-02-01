# core/logging Capability

## Purpose
Explain the logging expectations (startup visibility and structured failure insights) so operators know what runtime signals the bridge must emit.

## Requirements

### Requirement: Startup logging records runtime configuration
The bridge SHALL emit an info-level log once at startup that lists the resolved health forwarding flag, the sanitized Pixoo base URL (scheme and host only), the listener address, and the binary version so operators know what settings the container began with and which artifact they deployed.

#### Scenario: Container starts with health forwarding enabled
- **WHEN** the service finishes building `AppState` or equivalent and before it accepts HTTP traffic
- **THEN** it logs an info entry containing `health_forward=true`, the sanitized base URL, and the listener address

### Requirement: Unexpected Pixoo errors are logged with context
The bridge SHALL log every unexpected Pixoo interaction that results in an error (HTTP failures, invalid responses, non-zero `error_code`) at error level, including `error_code`, HTTP status if present, and any retriable flag so failures surface in container logs.

#### Scenario: Pixoo command fails with server error
- **WHEN** a command POST returns an HTTP 500 and retries are exhausted
- **THEN** the bridge logs an error entry with `status=500`, `retriable=true`, and the payload that triggered the failure

#### Scenario: Pixoo response reports non-zero `error_code`
- **WHEN** the Pixoo device responds with `error_code` ≠ 0
- **THEN** the bridge logs an error entry that includes the reported `error_code` and the remaining response payload so operators can correlate the device failure with the log

#### Scenario: Health check fails with client error
- **WHEN** the `/health` handler sees `PixooClient::health_check` return an error (e.g., HTTP 503)
- **THEN** it logs an error entry describing the failure before returning `503 SERVICE_UNAVAILABLE` to the caller
