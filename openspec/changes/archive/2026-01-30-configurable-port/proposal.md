## Why

The HTTP bridge currently binds to a hard-coded `8080`, which clashes with other services, containers, and deployment environments that expect more flexible port selections. Introducing a configurable listener ensures operators can satisfy local networking constraints immediately (defaulting to the lower `4000` range) while keeping the service lightweight.

## What Changes

- Add `PIXOO_BRIDGE_PORT` as the canonical way to override the HTTP listener port, validating/securing the value and defaulting to `4000` when omitted.
- Propagate the new configuration into the `Dockerfile`/README so containerized deployments can align the port mapping with the bridge config without editing source code.
- Document and test the default behavior so future contributions know the intended port strategy and can keep the HTTP layer simple.

## Capabilities

### New Capabilities
- `configurable-port`: Expose the HTTP listener port through `PIXOO_BRIDGE_PORT`, defaulting to `4000` and ensuring the bound port is valid before starting the Axum server; covers configuration loading, logging, and deployment guidance for the HTTP API domain (`core`).

### Modified Capabilities
- None.

## Impact

- `src/main.rs` (port binding and configuration parsing), plus any helpers/tests that rely on the listener address.
- `Dockerfile`, `README.md`, and other documentation so deployers know the new env var and default socket.
- adapt `openspec/specs/core/spec.md` defining requirements for the configurable HTTP surface.
