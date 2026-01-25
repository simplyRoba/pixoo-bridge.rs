## Context
The repo currently lacks any Cargo structure or CI automation, which means contributors cannot run safe builds or publish releases. The Pixoo bridge must start from a small Rust crate and use a predictable release pipeline so downstream automation can trust each version.

## Goals / Non-Goals
- **Goals:**
  - Provide a runnable Rust crate scaffold that compiles with the requested elemental dependencies.
  - Attach three separate GitHub Actions workflows: CI for testing, release-please for versioning, and Docker publishing for container distribution.
- **Non-Goals:**
  - Implement full Pixoo command set or business logic beyond the initial HTTP stub.
  - Replace release-please with a different release process.

## Decisions
- **Decision:** Use the standard `Cargo.toml` layout with `src/main.rs`/`src/lib.rs` entry points so the bridge behaves like a normal Rust executable and can host future modules (transport, framing, HTTP). This keeps the learning curve low for contributors and works with existing tooling.
- **Decision:** Depend on elemental crates such as `tokio`, `axum`, `serde`, and `thiserror` to cover async runtime, HTTP, serialization, and error handling. These are mature crates that satisfy the Pixoo bridge's needs without adding heavy frameworks.
- **Decision:** Create three separate GitHub Actions workflows: `ci.yml` for testing on PRs/pushes, `release-please.yml` for release-please to manage versioning and GitHub releases on main merges, and `publish-release.yml` to publish Docker images to GHCR after releases are created. This separation keeps concerns clear and allows independent triggering.

## Risks / Trade-offs
- **Risk:** Defining dependencies now might leak assumptions about future modules; Mitigation: keep modules abstract and add features only when needed.
- **Risk:** The Docker publishing workflow requires GHCR credentials; Mitigation: rely on GitHub's built-in `GITHUB_TOKEN` for publishing to the repository's container registry and document any required permissions.

## Migration Plan
- No existing structure to migrate; once this change lands, subsequent work can build on the new layout.

## Open Questions
- Should the workflow build multi-arch Docker images or stay single-arch for now? We can revisit after initial release.
