## Context

The bridge currently only sends Pixoo commands via HTTP POST and exposes no healthcheck endpoint for Docker. Pixoo devices provide a single undocumented HTTP GET `/get` endpoint that can indicate device health, but it does not follow the normal command flow and may behave inconsistently. We need a bridge-level healthcheck that can optionally cascade to the device while keeping the HTTP layer thin and explicit.

## Goals / Non-Goals

**Goals:**
- Add a bridge healthcheck endpoint that can be used by container health probes.
- Allow optional forwarding to the Pixoo `/get` endpoint controlled by `PIXOO_BRIDGE_HEALTH_FORWARD` (default true).
- Keep Pixoo HTTP behavior explicit, typed, and minimal in dependencies.

**Non-Goals:**
- No new Pixoo command features beyond the healthcheck GET call.
- No background polling or caching of health status.
- No changes to the existing command POST flow or retry behavior outside healthchecks.

## Decisions

- **Expose a simple bridge health route**: Add a lightweight endpoint (e.g., `GET /health`) that returns `200` when the bridge is healthy. If forwarding is enabled, it performs a Pixoo GET and propagates failures as non-200. This keeps Docker healthcheck integration straightforward without adding new dependencies.
  - **Alternative**: Use `GET /` or reuse existing routes. Rejected because it risks conflating command routes with health checks and complicates routing.

- **Introduce a dedicated Pixoo GET call in the client**: Add an explicit method on the Pixoo client to call `/get` and parse the response. This keeps the unusual GET path isolated and avoids mixing it with command payload logic.
  - **Alternative**: Implement the GET in the HTTP handler directly. Rejected to keep HTTP interaction encapsulated in the Pixoo client and reuse error handling patterns.

- **Config gate for cascading**: Use `PIXOO_BRIDGE_HEALTH_FORWARD` (default true) to control whether the bridge healthcheck cascades to the device. When disabled, the bridge returns healthy without contacting Pixoo.
  - **Alternative**: Default to false and only opt-in. Rejected because the purpose is to provide device health, and most deployments will want it.

- **Minimal retry/backoff**: Reuse existing retry policy for the client when hitting `/get`, but avoid adding a separate retry layer in the handler. This keeps the HTTP layer thin while still benefiting from standard transient handling.

## Risks / Trade-offs

- **Pixoo `/get` instability** → Mitigation: rely on existing retry policy and allow forwarding to be disabled via `PIXOO_BRIDGE_HEALTH_FORWARD`.
- **Healthcheck latency under retries** → Mitigation: keep retry count small and consistent with existing client defaults; document behavior in specs.
- **Undocumented endpoint semantics** → Mitigation: treat non-200 or invalid responses as unhealthy and keep parsing strict.
