## MODIFIED Requirements

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
