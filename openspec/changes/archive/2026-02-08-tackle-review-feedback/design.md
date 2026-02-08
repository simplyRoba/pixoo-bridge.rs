## Context

The bridge currently constructs `AppState` with an optional Pixoo client and tolerates missing `PIXOO_BASE_URL`, causing runtime 503 responses and repetitive handler guards. Configuration is partially read inside the Pixoo client module via environment access, which makes behavior inconsistent and harder to test. The Pixoo API is flaky, so retries/backoff should stay in the client, while the HTTP layer remains thin. This design aligns with the core/configuration capability defined in the change.

## Goals / Non-Goals

**Goals:**
- Fail fast on missing required configuration (`PIXOO_BASE_URL`) during startup.
- Load configuration once at startup and pass explicit values into client construction.
- Remove the optional client pattern from `AppState` and handlers.
- Keep retry/backoff logic centralized in the Pixoo client while keeping handler logic minimal.

**Non-Goals:**
- Changing external HTTP endpoints or response formats beyond startup failure behavior.
- Reworking Pixoo protocol framing or retry algorithms.
- Introducing new runtime dependencies or config formats.

## Decisions

- **Typed configuration loader in startup**: Introduce a `Config` (or similar) struct that parses environment variables once, validates required fields, and exposes typed values (URL, timeouts, retries/backoff). This makes startup failure explicit and moves global state access out of the Pixoo client.
  - *Alternatives*: Keep ad-hoc `env::var` access in client constructors; rejected because it scatters configuration and complicates tests.

- **Non-optional Pixoo client in `AppState`**: Replace `Option<PixooClient>` with a concrete client instance created during startup. This eliminates per-handler guard boilerplate and aligns with the fail-fast goal.
  - *Alternatives*: Keep `Option` and add a helper; rejected because it still permits an unusable runtime state.

- **Explicit client construction parameters**: Update `PixooClient::new` (or add a builder) to accept timeout and retry settings explicitly from `Config`. This keeps client behavior deterministic per process and avoids runtime env drift.
  - *Alternatives*: Read env variables in `client_timeout`; rejected due to implicit, per-construction global reads and test friction.

## Risks / Trade-offs

- Startup now fails if `PIXOO_BASE_URL` is missing → Mitigation: provide clear error messages and document required env vars.
- Configuration refactor touches startup and handlers → Mitigation: keep changes localized to config loading and state wiring; avoid API shape changes beyond required startup behavior.
- If existing deployments rely on 503 behavior during misconfig → Mitigation: treat as **BREAKING** per proposal and communicate in release notes.
