# api/health Capability

## Purpose
The `/health` capability is moving under `api/system` so that system-level endpoints remain grouped for monitoring and maintenance.

## REMOVED Requirements

### Requirement: Bridge health endpoint
**Reason**: `/health` is now served from the `api/system` capability alongside other system hooks so operators interact with a single domain.
**Migration**: Use `api/system` to observe health; the response shape remains `{ "status": "ok" }`.

### Requirement: Forwarded health behavior
**Reason**: Health forwarding is still provided but now belongs inside `api/system` to reuse the same middleware pipeline as `/reboot`.
**Migration**: See `api/system`’s health requirements for the forwarding expectations (unchanged behavior).
