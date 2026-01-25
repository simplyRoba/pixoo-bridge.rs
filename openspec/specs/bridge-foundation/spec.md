# bridge-foundation Specification

## Purpose
TBD - created by archiving change add-foundation-ci-flow. Update Purpose after archive.
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

### Requirement: CI workflow for testing and building
A GitHub Actions workflow located at `.github/workflows/ci.yml` SHALL run formatting/tidiness checks and `cargo test` on every pull request and push, and build release artifacts on pushes to `main` without publishing containers or releases.

#### Scenario: Pull request validation
- **WHEN** a pull request targets `main`
- **THEN** the workflow checks out the code, sets up Rust, runs `cargo fmt -- --check` and `cargo test`, and reports success or failure back to the PR.

#### Scenario: Push builds artifacts
- **WHEN** code is pushed to `main`
- **THEN** the workflow builds the project (`cargo build --release`) and stores artifacts without publishing containers or creating releases.

### Requirement: Release workflow for versioning
A GitHub Actions workflow located at `.github/workflows/release-please.yml` SHALL run release-please to create release branches and GitHub releases automatically when commits merge to `main`.

#### Scenario: Automated release creation
- **WHEN** a commit merges into `main`
- **THEN** release-please creates a release branch, drafts a GitHub release with version tag, and updates the CHANGELOG.

### Requirement: Docker publishing workflow
A GitHub Actions workflow located at `.github/workflows/publish-release.yml` SHALL publish the Docker image to GHCR after a GitHub release is published.

#### Scenario: Container publishing on release
- **WHEN** a GitHub release is published
- **THEN** the workflow builds the Docker image and pushes it to GHCR with the release tag.

