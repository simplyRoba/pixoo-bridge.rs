# core/server Capability

## Purpose
Define server lifecycle behavior including startup and graceful shutdown to ensure reliable operation in containerized environments.

## Requirements

### Requirement: Server handles shutdown signals gracefully
The bridge SHALL listen for `SIGTERM` and `SIGINT` signals and initiate graceful shutdown when received, stopping acceptance of new connections while allowing in-flight requests to complete.

#### Scenario: SIGTERM received during idle
- **WHEN** the server receives `SIGTERM` with no active requests
- **THEN** the server stops the listener and exits with code 0

#### Scenario: SIGTERM received with active requests
- **WHEN** the server receives `SIGTERM` while requests are being processed
- **THEN** the server stops accepting new connections
- **AND** waits for in-flight requests to complete before exiting

#### Scenario: SIGINT received (Ctrl+C)
- **WHEN** the server receives `SIGINT`
- **THEN** the server initiates the same graceful shutdown as `SIGTERM`

### Requirement: Server supports Unix and non-Unix platforms
The bridge SHALL handle `SIGTERM` on Unix platforms and gracefully degrade to `SIGINT`-only on non-Unix platforms.

#### Scenario: Running on Linux/macOS
- **WHEN** the bridge is compiled for a Unix target
- **THEN** both `SIGTERM` and `SIGINT` trigger graceful shutdown

#### Scenario: Running on Windows
- **WHEN** the bridge is compiled for Windows
- **THEN** only `SIGINT` (Ctrl+C) triggers graceful shutdown
- **AND** the bridge compiles without errors
