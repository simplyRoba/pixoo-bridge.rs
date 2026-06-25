## MODIFIED Requirements

### Requirement: CI workflow for testing and building
A GitHub Actions workflow located at `.github/workflows/ci.yml` SHALL run formatting/tidiness checks and `cargo test` on every pull request and push, and SHALL NOT build or upload release binaries because those artifacts are now compiled and published by the release workflow.

#### Scenario: Pull request validation
- **WHEN** a pull request targets `main`
- **THEN** the workflow checks out the code, restores the shared cache, and runs `cargo fmt -- --check` plus `cargo clippy -- -D warnings` followed by `cargo test`
- **THEN** no release binaries are built or uploaded in this workflow so the run remains fast and focused on lint/test feedback.

#### Scenario: Push runs stay focused on tests
- **WHEN** code is pushed to `main`
- **THEN** the workflow again runs the formatting checks and `cargo test`
- **THEN** the workflow explicitly avoids producing release binaries, leaving release compilation and publishing to the release workflow that listens for the GitHub release event.

### Requirement: Docker publishing workflow
A GitHub Actions workflow located at `.github/workflows/publish-release.yml` SHALL publish the Docker image to GHCR after a GitHub release is published, SHALL be split into scoped jobs that cover setup (installing compilers/targets), compilation of each binary using the same commands the Dockerfile expects, uploading those binaries as release assets, and building the Docker image from the precompiled files, and SHALL ensure each job’s `needs` relationship reflects that ordering.

#### Scenario: Container publishing on release
- **WHEN** a GitHub release is published
- **THEN** the workflow installs the cross compilers, adds the required targets, builds each binary with the identical linker/count settings used in the Dockerfile, uploads the compiled binaries as release assets named `pixoo-bridge-rs-x86_64` and `pixoo-bridge-rs-aarch64`, and then builds the Docker image that copies the correct binary based on `TARGETPLATFORM`
- **THEN** the workflow pushes the multi-arch image to GHCR once both binaries are verified and the release assets are attached.
