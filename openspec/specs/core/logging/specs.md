# core/logging Capability

## Purpose
Explain the logging expectations (startup visibility and structured failure insights) so operators know what runtime signals the bridge must emit.

## Requirements

### Requirement: Startup logging records runtime configuration
The bridge SHALL emit an info-level log once at startup that lists the resolved health forwarding flag, the Pixoo base URL, the listener address, and the binary version so operators know what settings the container began with and which artifact they deployed.

#### Scenario: Container starts with health forwarding enabled
- **WHEN** the service finishes building `AppState` or equivalent and before it accepts HTTP traffic
- **THEN** it logs an info entry containing `health_forward=true`, the configured base URL, and the listener address

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

### Requirement: Pixoo command interactions are traceable via debug logs
The bridge SHALL emit DEBUG logs for every Pixoo command, recording the command name and serialized payload before sending and logging the parsed response after a successful round trip so operators can correlate outbound requests with device responses even when no error occurs.

#### Scenario: Pixoo command succeeds
- **WHEN** the bridge forwards a validated command to Pixoo and receives a response with `error_code=0`
- **THEN** it logs a DEBUG entry containing the command name, the arguments used, and the response payload that was parsed from Pixoo

#### Scenario: Pixoo command is about to be retried
- **WHEN** a Pixoo command is replayed because the first attempt failed with a retriable error
- **THEN** a DEBUG log precedes the retry with the same command name and arguments so operators can see the retry volume

### Requirement: Access log records each request at DEBUG level
The bridge SHALL emit an access log entry at DEBUG level for every inbound HTTP request that includes the HTTP method, request path, response status code, and duration so operators can trace traffic without per-endpoint instrumentation.

#### Scenario: Successful request
- **WHEN** any client request is handled successfully and the bridge responds with `2xx`
- **THEN** a DEBUG log entry records that request’s method, path, status code, and duration, showing that the request completed normally

#### Scenario: Failed request
- **WHEN** a request cannot be fulfilled because Pixoo rejects it or an internal error occurs and the bridge returns a non-`2xx` status
- **THEN** a DEBUG log entry still records the method, path, status code, and duration so the failure is visible in the access log

### Requirement: Logs contain request identifiers
Logging SHALL associate every entry emitted during an HTTP request with the generated `X-Request-Id` value so operators can trace a request from ingress through Pixoo interactions.

#### Scenario: Successful request flow
- **WHEN** a request is handled, validated, and forwarded to Pixoo without errors
- **THEN** every log entry triggered during that request (route handler, helper functions, Pixoo client, and the HTTP response middleware) includes `request_id=<X-Request-Id value>` and the generated id is echoed in the response header

#### Scenario: Pixoo command failure
- **WHEN** a Pixoo command returns an error or `error_code` ≠ 0 after retries
- **THEN** the error log entry includes `request_id=<X-Request-Id value>` so operators can join the Pixoo failure with the originating HTTP request

#### Scenario: Request is rejected before reaching Pixoo
- **WHEN** validation fails in a handler and the request is short-circuited with a 4xx response
- **THEN** the log entry for the validation failure still includes `request_id=<X-Request-Id value>` and the response echoes that header so clients observing both can correlate the failure

### Requirement: Shutdown events are logged
The bridge SHALL log shutdown-related events at INFO level so operators can observe the shutdown lifecycle in container logs.

#### Scenario: Shutdown signal received
- **WHEN** the server receives a shutdown signal (`SIGTERM` or `SIGINT`)
- **THEN** an INFO log entry is emitted indicating the signal type and that shutdown is starting

#### Scenario: Shutdown complete
- **WHEN** all in-flight requests have completed and the server is about to exit
- **THEN** an INFO log entry is emitted indicating graceful shutdown completed
