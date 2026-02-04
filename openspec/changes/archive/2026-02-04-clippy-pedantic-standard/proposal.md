# Proposal: Clippy Pedantic as Project Standard

## Why

The codebase lacks consistent enforcement of Rust idioms and best practices. Code style varies across modules, and potential issues that clippy's pedantic lints would catch go unnoticed.

## What Changes

- Add `[lints.clippy]` configuration to `Cargo.toml` with pedantic enabled
- Fix all existing pedantic lint violations across the codebase
- Add `# Errors` documentation to public functions returning `Result`
- Selectively allow only style-preference lints (`must_use_candidate`, `module_name_repetitions`)

## Capabilities

### Modified Capabilities
- `core/foundation`: Add lint configuration and code standards requirements

## Impact

- `Cargo.toml`: Add `[lints.clippy]` section
- `src/pixoo/client.rs`: Fix casts, add error docs
- `src/pixoo/error.rs`: Fix match arms, wildcard patterns
- `src/routes/system.rs`: Fix let-else patterns
- `src/routes/manage.rs`: Fix let-else patterns
- `src/routes/tools.rs`: Fix let-else patterns, pass-by-reference
