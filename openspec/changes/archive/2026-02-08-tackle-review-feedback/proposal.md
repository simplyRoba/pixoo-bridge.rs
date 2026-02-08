## Why

The bridge currently starts without a Pixoo target and fails most requests at runtime, which hides misconfiguration and adds handler boilerplate. We need a fail-fast, explicit configuration load to make startup predictable and client construction testable.

## What Changes

- **BREAKING** Require `PIXOO_BASE_URL` at startup; the process exits with a clear error if missing instead of serving 503s.
- Load and validate Pixoo client configuration (URL, timeout, retries/backoff if applicable) once at startup and pass it explicitly to client construction.
- Remove the optional client pattern from `AppState` so handlers no longer guard against `None`.

## Capabilities

### New Capabilities

### Modified Capabilities
- `configuration`: Tightens startup requirements for `PIXOO_BASE_URL` and adds explicit startup-time client configuration behavior.

## Impact

- Startup/config loading in `main` and any config module.
- `AppState` construction and handler signatures that currently expect `Option<PixooClient>`.
- Pixoo client construction and timeout configuration in `src/pixoo/client.rs`.
- Operational behavior for deployments missing `PIXOO_BASE_URL`.
