# core/foundation Capability

## ADDED Requirements

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
