## Why

The bridge exposes a growing HTTP surface (draw, tools, manage, system) but offers no machine- or human-readable API contract. Integrators (Home Assistant, scripts) must read source code to discover routes, payload shapes, and error responses. An OpenAPI specification plus an interactive Swagger UI makes the API self-documenting and testable, lowering the integration cost for end users.

## What Changes

- Generate an OpenAPI 3.1 document from the existing Axum routes using `utoipa` together with the `utoipa-axum` integration, so route registration and documentation stay in one place and cannot drift.
- Serve the interactive Swagger UI at the application **root path (`/`)**, backed by the generated spec at `/api-docs/openapi.json`.
- Annotate every existing endpoint (draw, tools, manage, system — including path-parameter and multipart routes) with `#[utoipa::path(...)]`, documenting method, path, parameters, request bodies, success responses, and the shared error shapes (400/413/502/503/404).
- Derive `ToSchema` on all request models (already partly done in the working tree) and on the response models that currently lack it (`ManageSettings`, `ManageTime`, `ManageWeather`).
- Convert the `mount_*` route helpers to the `utoipa-axum` `OpenApiRouter` / `routes!` pattern so documented paths are collected automatically.
- Remove the dead `use utoipa::OpenApi` import in `manage/display.rs`.

Non-goals (out of scope): no changes to endpoint behavior, payloads, validation, or error semantics; no authentication on the docs UI; no client SDK generation.

## Capabilities

### New Capabilities
- `api-docs`: Generation and serving of the OpenAPI specification and the interactive Swagger UI, including the requirement that documentation is sourced from the live router so it stays in sync with implemented routes.

### Modified Capabilities
<!-- None. Endpoint behavior and validation are unchanged; documentation is additive. -->

## Impact

- **Dependencies**: keep `utoipa` and `utoipa-swagger-ui` (already declared); add `utoipa-axum` for drift-free route collection.
- **Code**:
  - New `src/openapi.rs` defining the top-level `#[derive(OpenApi)]` document (title/version/info) and the Swagger UI mount.
  - `src/main.rs` / `src/routes/mod.rs`: mount the Swagger UI at `/` and assemble routes via `OpenApiRouter`.
  - `src/routes/{draw,tools,system}.rs` and `src/routes/manage/{mod,display,time,weather}.rs`: `#[utoipa::path]` annotations and missing `ToSchema` derives.
- **Routing**: Swagger UI served at `/`; the spec at `/api-docs/openapi.json`. The existing `/health` and all other routes are unaffected. Port and bind address unchanged (default 4000).
- **Pixoo constraints**: Purely an HTTP-documentation layer; no new Pixoo device interaction, so flaky-device concerns do not apply.
