## Context

The bridge is a thin Axum HTTP layer over the Pixoo device. Routes are composed in `src/routes/mod.rs::mount_all_routes`, which chains module-level `mount_*` helpers (`draw`, `tools`, `manage`, `system`); the resulting router is wrapped with middleware in `src/main.rs::build_app` and served on `0.0.0.0:{listener_port}` (default 4000). Request models already derive `Deserialize` + `Validate`; a partial, untracked working-tree effort already added `#[derive(ToSchema)]` to several request structs and declared `utoipa` + `utoipa-swagger-ui` in `Cargo.toml`, but nothing is wired up (no `#[utoipa::path]`, no aggregate doc, no UI mount, plus a dead `use utoipa::OpenApi` in `manage/display.rs`).

This change completes that work as a tracked, additive documentation layer. There is no Pixoo device interaction involved, so the project's usual flaky-device/backoff concerns do not apply here.

## Goals / Non-Goals

**Goals:**
- Generate an OpenAPI 3.x document directly from the live router so it cannot drift from implemented routes.
- Serve interactive Swagger UI at `/docs` (with `/` redirecting to it) and the raw spec at `/api-docs/openapi.json`.
- Document every endpoint (draw, tools, manage, system), including path-parameter and multipart routes, plus shared error responses.

**Non-Goals:**
- No change to endpoint behavior, payloads, validation, or error semantics.
- No authentication/authorization on the docs UI.
- No client SDK or code generation.

## Decisions

### Decision: Use `utoipa-axum` (`OpenApiRouter` + `routes!`) instead of a hand-maintained `#[derive(OpenApi)] paths(...)` list
Each `mount_*` helper is converted to build an `OpenApiRouter<Arc<AppState>>` and register handlers with `routes!(handler)`. Paths are collected automatically as routes are mounted.

- **Why**: The classic approach requires a central `paths(...)` registry that must be manually kept in sync with the router — a known drift hazard. `utoipa-axum` keeps registration and documentation in a single call site, mirroring the existing `mount_*` structure, satisfying the "generated from the live router" requirement structurally rather than by discipline.
- **Alternative considered**: Classic `#[derive(OpenApi)]` with explicit `paths(...)`/`components(schemas(...))`. Rejected: avoids one dependency but reintroduces drift and duplicates the route list.
- **Cost**: Adds the `utoipa-axum` crate (small, pure-Rust, no heavy transitive deps).

### Decision: Mount Swagger UI at `/docs` with a `/` → `/docs` redirect
Use `utoipa_swagger_ui::SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi())`, merged into the app in `build_app`, plus a `GET /` route that issues a permanent redirect to `/docs`.

- **Why**: Operators still reach docs from the host root (via the redirect) without a path to remember, while functional routes stay reachable.
- **Why not root `/` directly**: Mounting Swagger UI at `/` makes `utoipa-swagger-ui` register a catch-all `/{*rest}` route that intercepts every unknown path and returns its own empty 404, clobbering the bridge's existing JSON `{"error":"not found"}` fallback (an established, tested contract). Mounting at `/docs` confines the catch-all to the docs subtree and preserves the JSON 404.
- **Trade-off**: The root path is a redirect rather than a landing page. Acceptable: the bridge is an API, and `/health` and all functional routes are unaffected.

### Decision: Centralize only the top-level document in a new `src/openapi.rs`
`src/openapi.rs` holds the `#[derive(OpenApi)] struct ApiDoc` with `info` (title, version sourced from `CARGO_PKG_VERSION`, description) and any shared `components`/error schemas. Per-path docs stay next to their handlers via `#[utoipa::path]`.

- **Why**: Keeps the global metadata in one obvious place while colocating route docs with code, matching the repo's module-per-area layout.

### Decision: Document shared error shapes once and reference them
Add `ToSchema`-deriving structs (or documented inline schemas) for the common error bodies (validation 400, payload-too-large 413, device-unreachable 502, device-error 503) and reference them in `#[utoipa::path(responses(...))]` where applicable.

- **Why**: The error shapes are uniform across handlers; documenting them once keeps annotations concise and accurate.

## Risks / Trade-offs

- [Swagger UI asset paths could shadow other routes] → Mounting at `/docs` confines Swagger UI's catch-all to the `/docs` subtree; the bare `/` only serves a redirect, and the JSON 404 fallback handles all other unknown paths. New functional routes use existing namespaced prefixes (`/draw`, `/tools`, `/manage`, `/system`-style).
- [Annotation drift for response codes — a handler could return a status not listed] → Mitigated structurally for paths by `utoipa-axum`; response-code accuracy is covered by review and the verification step. Behavior is untouched, so risk is documentation-only.
- [Binary size / compile time from `utoipa-swagger-ui` embedded assets] → Accepted; the crate was already chosen in the WIP. If size becomes a concern, a follow-up can switch to JSON-spec-only. Out of scope here.
- [`preserve_order` feature already enabled] → Keeps schema field ordering stable/deterministic in the generated spec; no action needed.

## Migration Plan

1. Add `utoipa-axum` to `Cargo.toml`; keep `utoipa` and `utoipa-swagger-ui`.
2. Add missing `ToSchema` derives to response structs; remove the dead `OpenApi` import in `manage/display.rs`.
3. Add `#[utoipa::path]` to each handler.
4. Convert `mount_*` helpers to `OpenApiRouter` and aggregate them; build `ApiDoc` in `src/openapi.rs`.
5. Mount Swagger UI at `/docs`, redirect `/` → `/docs`, and serve the spec at `/api-docs/openapi.json` in `build_app`.
6. Run `cargo fmt`, `cargo clippy`, `cargo test`; add a test asserting `/api-docs/openapi.json` returns 200 with a valid document and that a sample route is present.

**Rollback**: The feature is additive; reverting the change removes the docs routes and dependency with no impact on existing endpoints.

## Open Questions

- None blocking. (Spec version is `info` version only and is decoupled from release-please tagging.)
