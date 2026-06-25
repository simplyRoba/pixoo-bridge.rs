## ADDED Requirements

### Requirement: Access log records each request at DEBUG level
The bridge SHALL emit an access log entry at DEBUG level for every inbound HTTP request that includes the HTTP method, request path, response status code, and duration so operators can trace traffic without per-endpoint instrumentation.

#### Scenario: Successful request
- **WHEN** any client request is handled successfully and the bridge responds with `2xx`
- **THEN** a DEBUG log entry records that request’s method, path, status code, and duration, showing that the request completed normally

#### Scenario: Failed request
- **WHEN** a request cannot be fulfilled because Pixoo rejects it or an internal error occurs and the bridge returns a non-`2xx` status
- **THEN** a DEBUG log entry still records the method, path, status code, and duration so the failure is visible in the access log
