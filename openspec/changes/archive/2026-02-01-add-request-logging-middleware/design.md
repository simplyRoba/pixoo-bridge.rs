## Context

The bridge already configures `tracing` globally with `tracing_subscriber::fmt()` and respects `PIXOO_BRIDGE_LOG_LEVEL`. However, the router today only emits logs when handlers explicitly call `info!`/`debug!`, so inbound request metadata is absent unless each handler adds instrumentation. A lightweight middleware built with `axum::middleware::from_fn` can fill the gap.

## Goals / Non-Goals

**Goals:**
- Emit structured logs for every incoming HTTP request (method, path, status, latency) without touching each handler.
- Keep the new dependency surface minimal while following existing `tracing` configuration so observability improves without bigger architectural shifts.
- Document the new behavior for operators so they know where to look when investigating request failures.

**Non-Goals:**
- Building a full observability stack (metrics, distributed tracing) or exposing the new middleware as a configurable feature flag.
- Rewriting the router architecture or consolidating the existing health/tools routing logic beyond the added logging layer.

## Decisions

1. **Add an `access_log` middleware via `from_fn`** – it captures method, path, status, and latency before passing requests to each handler, which means no route changes are required and the middleware sits at the top of the router stack.
2. **Emit the access log at DEBUG level** so the logs only appear when `PIXOO_BRIDGE_LOG_LEVEL=debug`, keeping normal deployments quiet while giving operators a consistent place to look when they need traffic traces.
3. **Document the behavior in the README** so operators know request logging exists, how to enable it, and that it always logs at DEBUG.

## Risks / Trade-offs

[Increased log volume] → The middleware emits entries for every request at DEBUG level, which could overwhelm logs if the operator mistakenly runs with `PIXOO_BRIDGE_LOG_LEVEL=debug` in production. Mitigation: the default level remains INFO so access logs are quiet until the operator explicitly switches to DEBUG.
[Minimal dependency impact] → The middleware uses axum’s built-in `from_fn`, so no extra dependencies beyond `axum` and `tower` are required.

## Migration Plan

1. Merge this change so the router always wraps with the access-log middleware and emits DEBUG entries.
2. Update release notes/README to mention that request logs now appear at DEBUG and point operators to `PIXOO_BRIDGE_LOG_LEVEL`.
3. Redeploy the Docker image so operators can enable DEBUG logging and see the new access entries.

## Open Questions

- Should we expose request logging level or format via a dedicated env var, or is relying on `PIXOO_BRIDGE_LOG_LEVEL` sufficient?
