## Why

In containerized environments (Docker, Kubernetes), the orchestrator sends `SIGTERM` to signal shutdown. Without handling this signal, in-flight HTTP requests are abruptly dropped during deployments or restarts, causing failed requests for clients. Graceful shutdown allows the server to stop accepting new connections while completing existing requests.

## What Changes

- Add signal handling for `SIGTERM` and `SIGINT` (Ctrl+C)
- Integrate with Axum's graceful shutdown mechanism to drain connections
- Log shutdown events for observability
- Optionally support a configurable shutdown timeout

## Capabilities

### New Capabilities
- `core/server`: Server lifecycle management including startup and graceful shutdown behavior

### Modified Capabilities
- `core/logging`: Add shutdown-related log events

## Impact

- **Code**: `src/main.rs` - add shutdown signal handler and wire into Axum serve
- **Dependencies**: Uses `tokio::signal` (already available via tokio)
- **Behavior**: Server now waits for in-flight requests before exiting
- **Container**: Deployments become zero-downtime when combined with proper health checks
