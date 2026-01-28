## 1. Logging implementation

- [x] 1.1 Log the sanitized runtime configuration (health forwarding, Pixoo base URL, listener address) when `main()` finishes building the shared state and before serving requests.
- [x] 1.2 Emit structured `error!` entries for every Pixoo client failure path (`execute_once`, `execute_health_once`, and any downstream call that returns non-zero `error_code`), tagging HTTP status, device `error_code`, retriable flag, and payload when available.
- [x] 1.3 Log the health handler failure reason before returning `503 SERVICE_UNAVAILABLE`, referencing the Pixoo error context so operators can triage downstream failures.
- [x] 1.4 Add the `PIXOO_BRIDGE_LOG_LEVEL` environment variable with a default of `info` as the source of truth for the runtime logging filter so operators can adjust verbosity without rebuilding.
- [x] 1.5 Emit `debug`-level logs for notable success cases (e.g., a successful health check when `health_forward` is true or the final successful attempt after retries) so operators can enable verbose tracing without cluttering standard logs.

## 2. Verification

- [x] 2.1 Expand or add unit tests to prove the Pixoo error flows still produce the expected results after logging (e.g., exercising POST failures and health checks) so logging changes stay covered.
- [x] 2.2 Update `README.md` (or another user-facing doc) to describe that the container now emits startup configuration logs and surfaces unexpected Pixoo errors in stderr. if necessary.
- [x] 2.3 Document the available log-level values (debug/info/warn/error) in the README so folks know what to set `PIXOO_BRIDGE_LOG_LEVEL` to when they need more or less verbosity.

## 3. Final checks

- [x] 3.1 Run `cargo fmt`, `cargo clippy`, and `cargo test` locally to confirm the logging changes compile cleanly.
