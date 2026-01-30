## Context

The bridge currently binds Axum to a hard-coded `SocketAddr` on port `8080`, which breaks deployments where that port is consumed, contradicts the lean Docker image goals, and forces contributors to edit `main.rs` or the container definition to place the service behind a configurable port. The rest of the stack (Pixoo client, health-forward flag, logging) already flows through `AppState`, so adding a single configuration knob for the listener port fits the existing architecture without touching Pixoo-specific framing logic.

## Goals / Non-Goals

**Goals:**
- Introduce `PIXOO_BRIDGE_PORT` as the canonical way to override the HTTP listener port while defaulting to `4000`.
- Validate that the supplied port is within the `1024..=65535` range and emit clear logs if fallback happens.
- Make the default port expectation explicit in documentation (`README`, `Dockerfile`, changelog) so operators know what to map in container runtimes.

**Non-Goals:**
- Adding dynamic discovery of free ports or binding to ephemeral ports; deployments should control this via the env var.
- Restructuring the HTTP routing or Pixoo client surface—those remain untouched.

## Decisions

- **Port configuration via env var**: `PIXOO_BRIDGE_PORT` will be read once during startup with `env::var`, falling back to `4000` for reproducibility; this keeps the change localized to `main.rs` and eliminates runtime mutation.
- **Validation before binding**: Parse the port as `u16`, ensure it sits within the user-space range, and log a warning plus default to `4000` when the value is missing or invalid. This avoids panics while still warning operators.
- **Logging context**: Extend the existing `info!` block to include the configured port, making it easier to verify deployments without diving into docs.
- **Docker/documentation updates**: Update `Dockerfile` to expose `4000` and the README to call out `PIXOO_BRIDGE_PORT`, keeping external artifacts aligned.

- [Risk] Invalid `PIXOO_BRIDGE_PORT` values could silently fall back to `4000`, masking misconfiguration. → Mitigation: log a warning with the invalid value so operators can fix it.
- [Risk] Changing the default port might require downstream systems to update their port mappings. → Mitigation: document the new default clearly and keep the previous behavior available via explicit env var.
- [Risk] Additional configuration logic could bloat `main.rs`. → Mitigation: keep parsing logic concise and focus on readability; no new crates are introduced.

1. Update `main.rs` to read `PIXOO_BRIDGE_PORT`, validate it, and bind the listener accordingly.
2. Adjust `Dockerfile` to expose `4000` and mention the env var; ensure CI/test instructions are aware of the new port.
3. Document the new default in `README.md` (and potentially CHANGELOG) so operators know the expected binding.
4. Run `cargo fmt`, `cargo clippy`, and `cargo test` to verify no existing behavior regresses.

## Open Questions

- Should we provide a health check or readiness probe example that references the configured port, or leave that to operators?
- Are there any deployment scripts (e.g., Compose or Kubernetes manifests) that need to accompany this capability, or should those be handled separately?
