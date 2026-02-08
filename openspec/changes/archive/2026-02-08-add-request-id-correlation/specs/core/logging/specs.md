## ADDED Requirements

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
