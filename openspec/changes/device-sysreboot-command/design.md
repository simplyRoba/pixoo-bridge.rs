## Context

The Pixoo bridge currently routes every HTTP endpoint from `main.rs`, and recent additions (health, status, config) make the binary harder to navigate. This change introduces the `api/system` capability to ship Pixoo's `Device/SysReboot` command and groups the `/reboot` and `/health` endpoints under a shared routing surface so the HTTP layer can stay thin while honoring Pixoo's flaky UDP behavior and the project's small Docker image constraint.

## Goals / Non-Goals

**Goals:**
- Introduce an `api/system` capability that sends `Device/SysReboot` via the existing command framing/retry helpers and exposes an empty `/reboot` endpoint aligned with current auth/metrics middleware.
- Split the HTTP routes to keep `main.rs` manageable by moving `/health` and `/reboot` into a dedicated `routes/system.rs` (or similar) module that can grow with future system endpoints.
- Preserve the reliability guarantees (retries, timeouts, telemetry) the bridge already applies to other Pixoo commands.
- Update documentation to describe the new endpoint and route structure without leaking implementation details.

**Non-Goals:**
- Adding other unrelated Pixoo commands or reworking authentication.
- Replacing the existing retry/backoff strategy for Pixoo UDP traffic.

## Decisions

- **Route organization**: Create a `routes/system` module that re-exports a `mount` function invoked from `main.rs`. This module owns both `/health` and `/reboot`, letting `main.rs` remain a thin startup file while keeping future system endpoints colocated. Alternatives like keeping all routes in `main.rs` were rejected because the file is already unwieldy and the new capability would add more branches.
- **Capability modeling**: Treat `/health` as a modified part of `api/system` so spec readers see system-related endpoints together. The new `api/system` capability will encapsulate both the empty `/reboot` handler and the existing health-check usage, ensuring the domain-level spec stays cohesive.
- **Command dispatch**: Extend `pixoo_bridge::pixoo::Command` (or equivalent command router) with a `DeviceSysReboot` variant that reuses the framing/backoff logic used by other Pixoo commands. The HTTP handler delegates to this capability so the UDP layer can remain thin while still logging/propagating errors consistently.
- **Error handling and retries**: Reuse the existing retry helper instead of implementing a bespoke loop. The `/reboot` handler will translate Pixoo errors into HTTP `503` responses with consistent logging, mirroring the current behavior for other system calls.
- **Documentation touches**: Document the new endpoint and the route split in the README so operators know where `/reboot` lives and how it relates to `/health`.

## Risks / Trade-offs

- [Risk] Adding a new route module increases the surface area briefly, which could introduce routing bugs if `main.rs` no longer mounts the module correctly. → Mitigate by keeping the `routes/system` mounting API small (`mount(SystemRoutes::new())`) and adding smoke tests for `/health` and `/reboot`.
- [Risk] Pixoo may intermittently drop UDP packets, making `/reboot` unreliable. → Mitigate by reusing the existing retry/backoff helpers, logging failures, and surfacing a 503/5xx response so callers can retry on their side.
- [Risk] `api/health` now depends on the `api/system` module, so any future changes must navigate the reorganized path. → Document the dependency to prevent accidental regressions.

## Migration Plan

1. Add the `routes/system.rs` module and export a `mount_system_routes` helper; update `main.rs` to call it instead of inline health/reboot handlers.
2. Extend the Pixoo command router with the `DeviceSysReboot` variant and reuse the existing framing/retry helpers to emit the UDP packet.
3. Implement the `/reboot` HTTP handler in `routes/system.rs`, applying shared middleware (auth, logging, timeout) and translating Pixoo errors to clear HTTP statuses.
4. Update documentation (README/CHANGELOG) to mention `/reboot` and the route split while pointing readers to the new module for future system endpoints.
5. Add integration/unit tests covering the `/reboot` handler’s happy path and failure translation.

## Open Questions

- Should the new `routes/system` module expose any helper for future system endpoints beyond `/health` and `/reboot` (e.g., configuration or diagnostics)?
- Do we need any automated smoke test for the route split to guard against forgetting to mount the module in `main.rs`?
