# core/foundation Capability

## Purpose
Describe the foundational Rust bridge layout and dependencies so downstream contributors understand the minimal crate structure and runtime stack that every capability builds on.

## Requirements

### Requirement: Rust bridge foundation layout
The repository SHALL define a Rust binary crate rooted at `Cargo.toml` with the canonical `src/main.rs` entry point and supporting modules so the bridge compiles to a runnable HTTP service stub without Pixoo-specific logic yet.

#### Scenario: Fresh checkout compiles
- **WHEN** a contributor clones the repository and runs `cargo check`
- **THEN** the manifest, entry point, and placeholder modules resolve and compile successfully, producing an executable that can be extended by later commits.

### Requirement: Elemental dependency set
The crate SHALL declare the minimal async/HTTP/serialization helpers (for example `tokio`, `axum`, `serde`, `serde_json`, `thiserror`, and any lightweight Pixoo framing helpers) so downstream code can focus on Pixoo-specific transports without wiring runtime plumbing repeatedly.

#### Scenario: Dependency graph resolves
- **WHEN** the developer runs `cargo fetch` or `cargo build`
- **THEN** the declared crates download, compile, and provide the async runtime plus serialization helpers needed by the bridge foundation (without introducing additional heavyweight frameworks).

### Requirement: Clippy pedantic lint enforcement
The project SHALL enforce clippy pedantic lints via `Cargo.toml` configuration so code follows consistent Rust idioms and best practices.

#### Lint Configuration
```toml
[lints.clippy]
pedantic = { level = "warn", priority = -1 }
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

### Requirement: Error documentation
Public functions returning `Result` SHALL include an `# Errors` section documenting when each error variant is returned.

#### Scenario: Error documentation format
- **WHEN** a public function returns `Result<T, E>`
- **THEN** the doc comment includes an `# Errors` section with `Returns [`ErrorType::Variant`] if condition occurs.` entries.
