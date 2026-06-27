# api/docs Capability

## Purpose
Generate and serve the bridge's OpenAPI specification and an interactive Swagger UI so integrators can discover routes, payloads, and error shapes without reading source. Documentation is sourced from the live router so it stays in sync with implemented behavior.

## Requirements

### Requirement: OpenAPI specification is generated from the live router
The bridge SHALL generate an OpenAPI 3.x document from the same route registration used to serve requests, using `utoipa` with the `utoipa-axum` integration, so that every route mounted into the application is reflected in the specification and documentation cannot silently drift from implemented behavior.

#### Scenario: Mounted route appears in the specification
- **WHEN** a handler is registered through the application's `OpenApiRouter` and carries a `#[utoipa::path]` annotation
- **THEN** the generated OpenAPI document includes that route's method, path, parameters, request body schema (if any), and documented responses

#### Scenario: Specification reflects the full route set
- **WHEN** the OpenAPI document is generated
- **THEN** it contains an entry for every documented draw, tools, manage, and system endpoint, including path-parameter and multipart routes

### Requirement: OpenAPI document is served as JSON
The bridge SHALL serve the generated OpenAPI document as JSON at `/api-docs/openapi.json` so that external tooling can consume the API contract programmatically.

#### Scenario: Spec endpoint returns the document
- **WHEN** a client sends `GET /api-docs/openapi.json`
- **THEN** the bridge responds with HTTP 200 and a JSON body that is a valid OpenAPI document describing the bridge's endpoints

### Requirement: Interactive Swagger UI is served with a root redirect
The bridge SHALL serve an interactive Swagger UI at `/docs`, backed by the OpenAPI document, and SHALL redirect the application root path (`/`) to `/docs`, so that operators can browse and exercise the API from a browser without external tooling while the JSON 404 contract for unknown routes is preserved.

#### Scenario: Docs path serves the documentation UI
- **WHEN** a client opens `GET /docs/` in a browser
- **THEN** the bridge serves the Swagger UI page, which loads the spec from `/api-docs/openapi.json`

#### Scenario: Root path redirects to the docs UI
- **WHEN** a client opens `GET /`
- **THEN** the bridge responds with a redirect to `/docs`

#### Scenario: Existing endpoints and 404 contract remain intact
- **WHEN** the Swagger UI is mounted
- **THEN** all previously documented endpoints (such as `/health`, `/draw/fill`, and `/manage/settings`) continue to respond with their existing behavior
- **AND** an unknown route still returns the JSON `{ "error": "not found" }` body with status 404

### Requirement: Documented schemas cover request and response models
The bridge SHALL derive OpenAPI schemas (`ToSchema`) for every request model and for response models returned as JSON (including `ManageSettings`, `ManageTime`, and `ManageWeather`), and SHALL document errors with a single canonical error-envelope schema referenced by every error response, so consumers can rely on the specification for both success and failure cases.

#### Scenario: Request and response schemas are present
- **WHEN** the OpenAPI document is generated
- **THEN** request bodies and JSON response bodies reference named component schemas with their documented field types and constraints

#### Scenario: Error responses reference one canonical envelope
- **WHEN** an endpoint can return validation (400), not-found (404), payload-too-large (413), device-unreachable (502), device-error (503), device-timeout (504), or internal (500) responses
- **THEN** those response statuses are documented for the relevant endpoints
- **AND** every error response references the single canonical error-envelope schema (root `error_status`, `error_kind`, `message`, with an optional nested `details` object) rather than separate `ValidationErrorBody` or `PayloadTooLargeBody` shapes

#### Scenario: Error schema example is coherent
- **WHEN** the canonical error-envelope schema is rendered in the documentation UI
- **THEN** it shows a consistent example whose `error_kind` matches the accompanying `details` (e.g. a `device-error` example with `details.error_code`), not an impossible combination
