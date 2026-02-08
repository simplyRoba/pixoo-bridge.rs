## Context

The bridge currently calls `axum::serve(listener, app).await?` which blocks until the process is killed. In containerized deployments, orchestrators send `SIGTERM` to initiate shutdown. Without handling this signal, in-flight requests are terminated abruptly, causing client failures during rolling updates.

Axum provides `with_graceful_shutdown()` which stops accepting new connections while allowing existing requests to complete.

## Goals / Non-Goals

**Goals:**
- Handle `SIGTERM` (container orchestrator) and `SIGINT` (Ctrl+C) signals
- Drain in-flight requests before exiting
- Log shutdown events for operator visibility

**Non-Goals:**
- Configurable shutdown timeout (keep it simple; Kubernetes has its own `terminationGracePeriodSeconds`)
- HTTP endpoint to trigger shutdown (signals are sufficient)
- Persisting state on shutdown (stateless bridge)

## Decisions

### Decision 1: Use `tokio::signal` for signal handling

**Choice:** `tokio::signal::ctrl_c()` and `tokio::signal::unix::signal(SignalKind::terminate())`

**Rationale:** Already available via tokio (no new dependencies). Platform-aware with `#[cfg(unix)]` for SIGTERM.

**Alternatives considered:**
- `ctrlc` crate: Additional dependency, less integration with tokio
- `signal-hook`: More powerful but overkill for this use case

### Decision 2: Use `tokio::select!` to await either signal

**Choice:** Single async function that resolves when either signal arrives

**Rationale:** Clean pattern that Axum's `with_graceful_shutdown` expects. Returns once any signal fires.

### Decision 3: No configurable timeout

**Choice:** Rely on Axum's built-in behavior (waits for all connections to close)

**Rationale:** Container orchestrators already enforce timeouts via `terminationGracePeriodSeconds`. Adding our own would duplicate functionality and risk premature termination.

## Risks / Trade-offs

**[Risk] Long-running requests block shutdown indefinitely**
→ Mitigation: Kubernetes will SIGKILL after grace period. All our handlers are short-lived (Pixoo commands have their own timeouts).

**[Risk] Windows doesn't support SIGTERM**
→ Mitigation: `#[cfg(unix)]` guard compiles out SIGTERM on non-Unix. SIGINT (Ctrl+C) still works everywhere.

**[Trade-off] No shutdown timeout means trusting external orchestration**
→ Acceptable: This is a containerized service; bare-metal deployments are not the target.
