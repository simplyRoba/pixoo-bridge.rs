## Context

The repo boots a tiny Axum HTTP bridge that already wires `tracing`/`tracing_subscriber` and exposes a health-check endpoint. Startup currently initializes the HTTP listener silently, while Pixoo client failures bubble up through `PixooError` but are never logged. Docker operators (and our own future observability work) need a clear picture of the configuration the container used on startup and why unexpected failures happen.

## Goals / Non-Goals

- **Goals:**
- Emit an info-level log at startup that states the resolved configuration (health forwarding setting, Pixoo base URL, and timeout policy) without leaking secrets.
- Log every unexpected error path with structured metadata (error kind, HTTP status, payload, Pixoo error code) so operators can debug failures using container logs instead of needing to reproduce flows.
- Keep logging dependencies minimal by building on the existing `tracing` instrumentation and the single `axum` binary crate.
- Allow operators to tune the verbosity via a `PIXOO_BRIDGE_LOG_LEVEL` environment variable that defaults to `info`.
- Surface notable success events at `debug` level (e.g., health checks or command retries that eventually succeed) so operators can opt-in to richer traces without cluttering standard logs.

**Non-Goals:**
- Instrumenting a full distributed tracing stack or shipping log aggregation infrastructure.
- Logging every successful health check or command response (only failures and startup events require new visibility).

## Decisions

1. **Reuse the current `tracing_subscriber::fmt` setup** instead of introducing new logging crates. We will enrich the `info!`/`error!` calls with structured fields (e.g., `health_forward`, `pixoo_base_url`, `status`, `error_code`). This keeps the run-time footprint small and avoids adding dependencies beyond what `main.rs` already uses.
2. **Capture startup configuration right after environment parsing**. `main()` already reads `PIXOO_BRIDGE_HEALTH_FORWARD` and `PIXOO_BASE_URL`; we will log those values (masking sensitive parts of the URL) together with the listener address. This gives operators a one-shot record of what the container considered its configuration.
3. **Surface errors where they originate in the Pixoo client** (`execute_once`, `execute_health_once`, and the `health` handler). Each `Err(PixooError)` will be accompanied by an `error!` log that annotates retriable vs. terminal cases, HTTP status, raw response, and device `error_code` when available. Health handler failures will also log why the bridge responded `SERVICE_UNAVAILABLE`.
4. **Sanitize logged configuration**: we will log the Pixoo host and scheme but omit query parameters or user info to avoid leaking credentials. If future requirements demand masking extra values, we can centralize sanitization before logging fields.
5. **Expose log level control**: map `PIXOO_BRIDGE_LOG_LEVEL` to the logging backend so we can upgrade/downgrade verbosity at runtime without rebuilding the container.
6. **Use debug logs for successes**: keep production verbosity clean by logging only failures and startup info at info/error levels, while allowing debug mode to mention successful health checks or retries that finished cleanly.
7. **Document README options**: update README.md to describe the supported log levels (e.g., debug/info/warn/error) so operators know what values they can set for `PIXOO_BRIDGE_LOG_LEVEL` and what behavior to expect.

## Risks / Trade-offs

- **[Risk]** Logging configuration could expose sensitive endpoints or keys if they are embedded in `PIXOO_BASE_URL`. **Mitigation:** only log the scheme and host (strip query/user info) and avoid printing raw tokens.
- **[Risk]** Additional logging may flood container logs under high failure volume. **Mitigation:** Restrict new logs to unexpected failures and startup events; normal successes remain quiet.
- **[Risk]** Structured logging fields may evolve (e.g., more metadata needed later). **Mitigation:** Keep field naming consistent (`health_forward`, `pixoo_base_url`, `error_code`) so future consumers can extend without rewriting earlier entries.

## Migration Plan

1. Update `src/main.rs` to log resolved configuration (health forwarding flag, sanitized Pixoo base URL, listener address) after the state is built and honor `PIXOO_BRIDGE_LOG_LEVEL` when configuring `tracing`.
2. Update Pixoo client routines to `error!` log when retries exhaust or non-200 HTTP statuses/invalid responses occur, attaching entity details from `PixooError`.
3. Adjust `health` handler logs so it records when the upstream Pixoo health check failed before returning `SERVICE_UNAVAILABLE`.
4. Run `cargo fmt`, `cargo clippy`, and `cargo test` to ensure logging changes compile and no behavior regressions occur. Validate the Docker image still boots with the new logs (manually if needed).

## Open Questions

- Should we also log unexpected successes (e.g., command responses) at debug level, or keep the focus strictly on startup and failures?
