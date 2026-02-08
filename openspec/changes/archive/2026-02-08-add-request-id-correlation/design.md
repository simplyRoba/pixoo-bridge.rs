## Context

The bridge already enforces strict configuration via `AppConfig`, and route handlers funnel command dispatch through `AppState::pixoo_client`. Logs currently describe what failed, but there is no shared identifier attached to HTTP requests, handlers, or Pixoo commands, so reproducing which Pixoo interaction originated from which HTTP request requires cross-referencing timestamps manually. We need to introduce correlation without bloating the runtime or deviating from the existing middleware stack.

## Goals / Non-Goals

**Goals:**
- Generate a request identifier per inbound HTTP request and expose it via `X-Request-Id` in responses so clients can correlate logs and responses.
- Pass that identifier through Axum extensions, Tracing spans, and bespoke logging helpers to ensure every log message related to that request carries the same value.
- Inject the identifier into Pixoo client interactions so device-side logs or retries include it.
- Document how operators can consume the identifier in logs and responses.

**Non-Goals:**
- Overhauling the current logging format or adding a full OpenTelemetry export.
- Changing the Pixoo API contract beyond emitting the correlation header and enriching log context.

## Decisions

- **Middleware Placement**: Implement a Tower middleware that runs early in the stack, generates (or forwards) `X-Request-Id`, stores it in extensions, and pushes it into the response header. Alternative: manually generate ids within each handler, but that would duplicate logic. Middleware keeps the logic centralized and ensures every request—even those rejected before reaching the handler—still gets an identifier.
- **Correlation Storage**: Use Axum's `Extension`/`RequestParts` to store a lightweight `RequestId(String)` newtype. Each handler and the Pixoo client can retrieve it via `State`/`Extension` accessors. Alternatives like thread-local storage would complicate tests and async contexts.
- **Tracing Integration**: Annotate route handlers and Pixoo client entry points with `#[tracing::instrument(skip(state))]` so spans automatically inherit the request id as a field via `tracing::Span::current().record`. For lower-level Pixoo client helpers that cannot easily be instrumented, manually create spans and record the identifier. This keeps the structured logs consistent without deeply refactoring the Pixoo module.
- **Pixoo Client Logging**: When `PixooClient::send_command` logs errors or command dispatches, include the request id (if available) so the resulting log entries can be mapped back to HTTP traffic. We will extend `map_pixoo_error` logging helpers to accept the identifier, defaulting to `Option<&str>`.
- **Header Propagation**: Middleware should respect incoming `X-Request-Id` headers to avoid generating new ids for clients that already provide one. This makes the feature compatible with upstream proxies that already assign ids.

## Risks / Trade-offs

- **Risk**: A missing correlation header could still happen if middleware is accidentally bypassed (e.g., health check routes). → Mitigation: mount the middleware as close to the router as possible and document which routes go through it; health checks can reuse the same handler and set a generated id.
- **Risk**: The extra logging field increases log payload size slightly. → Mitigation: The identifier is short (UUID/trace id) and appended only to structured logs, keeping memory impact minimal.
- **Risk**: Instrumenting every handler may make `tracing` spans too chatty. → Mitigation: We only add `#[instrument(skip(state))]` where it simplifies context propagation (handlers, dispatch helpers) and rely on existing log levels to control verbosity.
- **Risk**: Adding request-id handling in Pixoo client duplicates logic across command helpers. → Mitigation: Encapsulate the behavior within helper utilities and reuse them wherever Pixoo logs are emitted.

## Migration Plan

1. Create the request-id middleware and wire it into the router setup so responses echo `X-Request-Id` and extensions receive the value.
2. Add a `RequestId` newtype and helper methods to `AppState`/`extensions` for retrieving it inside handlers and the Pixoo client.
3. Annotate handler entry points and Pixoo client helpers with `#[tracing::instrument]`/manual spans that capture the identifier.
4. Update logging helpers and `map_pixoo_error` to include the identifier and adjust tests to assert the header/span behavior.
5. Document the observability improvements in the logging spec and README to inform operators about the new header/hint.
6. Run `cargo fmt`, `cargo clippy`, and `cargo test` to ensure no regressions.

## Open Questions

- Should we expose the request id via response bodies (e.g., JSON error payloads) in addition to headers, or is the header sufficient for the current consumers?
