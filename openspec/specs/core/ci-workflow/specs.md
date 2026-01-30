# core/ci-workflow Capability

## Requirements

### Requirement: CI workflow for testing and building
A GitHub Actions workflow located at `.github/workflows/ci.yml` SHALL run formatting/tidiness checks and `cargo test` on every pull request and push, and build release artifacts on pushes to `main` without publishing containers or releases.

#### Scenario: Pull request validation
- **WHEN** a pull request targets `main`
- **THEN** the workflow checks out the code, sets up Rust, runs `cargo fmt -- --check` and `cargo test`, and reports success or failure back to the PR.

#### Scenario: Push builds artifacts
- **WHEN** code is pushed to `main`
- **THEN** the workflow builds the project (`cargo build --release`) and stores artifacts without publishing containers or creating releases.
