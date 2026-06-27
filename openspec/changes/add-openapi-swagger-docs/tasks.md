## 1. Dependencies & Cleanup

- [x] 1.1 Add `utoipa-axum` to `Cargo.toml`; confirm `utoipa` (with `axum_extras`, `preserve_order`) and `utoipa-swagger-ui` (with `axum`) remain declared
- [x] 1.2 Remove the dead `use utoipa::OpenApi` import in `src/routes/manage/display.rs`
- [x] 1.3 Add `#[derive(ToSchema)]` to response structs missing it: `ManageSettings` (manage/mod.rs), `ManageTime` (manage/time.rs), `ManageWeather` (weather.rs)

## 2. Shared Error Schemas

- [x] 2.1 Define `ToSchema`-deriving documentation structs (or documented inline schemas) for the shared error bodies: validation (400), payload-too-large (413), device-unreachable (502), device-error (503), and not-found (404)
- [x] 2.2 Expose these schemas to the OpenAPI document via the aggregate `components`

## 3. Top-level OpenAPI Document

- [x] 3.1 Create `src/openapi.rs` with `#[derive(OpenApi)] struct ApiDoc` defining `info` (title, description, version from `CARGO_PKG_VERSION`) and shared `components`
- [x] 3.2 Register the `openapi` module in `src/main.rs`/`lib` module tree

## 4. Path Annotations

- [x] 4.1 Annotate draw handlers (`/draw/fill`, `/draw/upload` multipart, `/draw/remote`, `/draw/text`, `/draw/text/clear`) with `#[utoipa::path]` including request bodies, params, success and error responses
- [x] 4.2 Annotate tools handlers (`/tools/timer/start`, `/tools/timer/stop`, `/tools/stopwatch/{action}`, `/tools/scoreboard`, `/tools/soundmeter/{action}`) including path params
- [x] 4.3 Annotate manage handlers (settings, time get/set, weather get/set-location, timezone, time-mode, temperature-unit, display on/off, brightness, rotation, mirror, overclock, white-balance) including all path params
- [x] 4.4 Annotate system handlers (`GET /health`, `POST /reboot`)

## 5. Router Assembly via utoipa-axum

- [x] 5.1 Convert each `mount_*` helper in `draw.rs`, `tools.rs`, `system.rs`, and `manage/mod.rs` to build/return an `OpenApiRouter<Arc<AppState>>` registering handlers with `routes!`
- [x] 5.2 Update `src/routes/mod.rs::mount_all_routes` to aggregate the `OpenApiRouter`s and split into an Axum `Router` plus the collected `OpenApi`
- [x] 5.3 In `src/main.rs::build_app`, merge the collected paths into `ApiDoc`, mount Swagger UI at `/` and the spec at `/api-docs/openapi.json`, preserving existing middleware and `with_state`

## 6. Tests

- [x] 6.1 Add a test asserting `GET /api-docs/openapi.json` returns 200 with a valid OpenAPI JSON document
- [x] 6.2 Add a test asserting the generated document contains a representative set of routes (e.g. `/draw/fill`, `/manage/settings`, `/health`) with expected methods
- [x] 6.3 Add a test asserting `GET /` serves the Swagger UI and that existing endpoints (e.g. `/health`) still respond unchanged

## 7. Documentation & Verification

- [x] 7.1 Update `README.md` to mention the Swagger UI at `/` and the spec at `/api-docs/openapi.json` (end-user facing only)
- [x] 7.2 Run `cargo fmt`, `cargo clippy`, and `cargo test`; resolve all warnings and failures
