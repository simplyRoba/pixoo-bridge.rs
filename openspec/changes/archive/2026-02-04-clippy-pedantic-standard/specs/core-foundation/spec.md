# Delta Spec: core/foundation

## ADDED Requirements

### Requirement: Clippy pedantic lint enforcement

The project SHALL enforce clippy pedantic lints via `Cargo.toml` configuration so code follows consistent Rust idioms and best practices.

#### Scenario: Clippy configuration is defined
- **WHEN** examining the project's Cargo.toml
- **THEN** it contains the following clippy pedantic lint configuration:
  ```toml
  [lints.clippy]
  pedantic = { level = "deny", priority = -1 }
  must_use_candidate = "allow"
  module_name_repetitions = "allow"
  ```

#### Scenario: Clippy passes with pedantic lints
- **WHEN** a contributor runs `cargo clippy --all-targets -- -D warnings`
- **THEN** the build completes without warnings, enforcing pedantic lint standards.

### Requirement: Code standards

All code SHALL follow these patterns to pass clippy pedantic lints:
- Use `let ... else` for early returns instead of `match`
- Use safe integer conversions (`try_from`) instead of `as` casts
- Pass by reference when ownership is not needed
- Use explicit enum variants instead of wildcards in exhaustive matches
- Consolidate identical match arms

#### Scenario: Code follows pedantic patterns
- **WHEN** new code is added to the project
- **THEN** it follows the established patterns and passes clippy pedantic lints.

### Requirement: Error documentation

Public functions returning `Result` SHALL include an `# Errors` section documenting when each error variant is returned.

#### Scenario: Error documentation format
- **WHEN** a public function returns `Result<T, E>`
- **THEN** the doc comment includes an `# Errors` section with `Returns [`ErrorType::Variant`] if condition occurs.` entries.
