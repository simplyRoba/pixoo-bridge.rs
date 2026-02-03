## Why

Manual validation sprinkled through the tools routes is brittle and inconsistent. Converting the request handling to structured, validated payloads and enums keeps Pixoo command wiring focused on translation instead of defensive parsing, which makes the bridge easier to reason about and extend now that the tool surfaces are stable. Extending these guards once across the tools surface prevents future handlers from regressing to ad-hoc parsing.

## What Changes

- Introduce shared request models for all tools handlers that derive `Deserialize` and `Validate`, using attribute-based ranges so invalid payloads are rejected before dispatching to Pixoo.
- Replace tool action path parameters with enums that serde deserializes with `lowercase` variants and reject unsupported values automatically.
- Expand the tools handlers and their tests to use the new models so Pixoo commands always receive normalized data while keeping the existing JSON responses unchanged.

## Capabilities

### New Capabilities
- `core/request-validation`: Define shared request models for every tool handler and their validation rules so only normalized data reaches the Pixoo command wiring.
- `api/common`: Define how tool action paths deserialize via lowercase enums and how handlers respond to unsupported actions.

### Modified Capabilities
- (none)

## Impact

- Updates `src/routes/tools.rs` to use the shared validated request structs and enum extractors so handlers only see normalized data.
- Adds serde and validator dependencies (or equivalent validation helpers) so the request models can encode their constraints once for all tools.
- Adjusts the tool route tests to cover the general validation paths and removes duplicate range/action matching logic from the handlers.
