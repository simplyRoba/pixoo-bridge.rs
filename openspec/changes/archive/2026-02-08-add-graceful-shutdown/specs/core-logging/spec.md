## ADDED Requirements

### Requirement: Shutdown events are logged
The bridge SHALL log shutdown-related events at INFO level so operators can observe the shutdown lifecycle in container logs.

#### Scenario: Shutdown signal received
- **WHEN** the server receives a shutdown signal (`SIGTERM` or `SIGINT`)
- **THEN** an INFO log entry is emitted indicating the signal type and that shutdown is starting

#### Scenario: Shutdown complete
- **WHEN** all in-flight requests have completed and the server is about to exit
- **THEN** an INFO log entry is emitted indicating graceful shutdown completed
