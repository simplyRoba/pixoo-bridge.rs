# core Specification

## Purpose
Define the Pixoo bridge's `core` domain by combining the HTTP client behavior required to talk to Pixoo devices with the foundational Rust runtime, dependency, and automation plumbing that downstream services rely on.

## Requirements

### Requirement: Command payload construction
The client SHALL construct a JSON request body that includes the `Command` field derived from a command enum plus all provided argument fields flattened into the same JSON object.

#### Scenario: Command with additional fields
- **WHEN** the caller issues a `Tools/SetTimer` command with `Minute`, `Second`, and `Status` arguments
- **THEN** the client sends a JSON object containing `Command`, `Minute`, `Second`, and `Status` in the request body

### Requirement: HTTP request shape
The client SHALL send Pixoo commands via HTTP POST to a configured device IP and set the request `Content-Type` to `application/json`. The client SHALL send Pixoo health checks via HTTP GET to the device `/get` endpoint without a request body.

#### Scenario: Post command to device
- **WHEN** the caller sends any Pixoo command
- **THEN** the client issues an HTTP POST to the configured device endpoint with `Content-Type: application/json`

#### Scenario: Get health from device
- **WHEN** the caller requests a Pixoo health check
- **THEN** the client issues an HTTP GET to the device `/get` endpoint

### Requirement: Response parsing with incorrect content type
The client SHALL parse the response body as JSON regardless of the response `Content-Type` header value.

#### Scenario: Response labeled text/html
- **WHEN** the device responds with `Content-Type: text/html` and a JSON body
- **THEN** the client parses the body as JSON and makes the fields available to the caller

### Requirement: Error code validation
The client SHALL read `error_code` from every response and treat any non-zero value as a failure.

#### Scenario: Device returns error
- **WHEN** the device responds with `error_code` set to a non-zero value
- **THEN** the client returns an error that includes the `error_code`

### Requirement: Successful response shaping
On successful responses (`error_code` equals zero), the client SHALL return the remaining response fields without `error_code`.

#### Scenario: Get command response fields
- **WHEN** the device responds with `error_code: 0` plus additional fields such as `Brightness` and `RotationFlag`
- **THEN** the client returns a response map containing the additional fields and omits `error_code`

### Requirement: Rust bridge foundation layout
The repository SHALL define a Rust binary crate rooted at `Cargo.toml` with the canonical `src/main.rs` entry point and supporting modules so the bridge compiles to a runnable HTTP service stub without Pixoo-specific logic yet.

#### Scenario: Fresh checkout compiles
- **WHEN** a contributor clones the repository and runs `cargo check`
- **THEN** the manifest, entry point, and placeholder modules resolve and compile successfully, producing an executable that can be extended by later commits.

### Requirement: Container healthcheck
The Docker image SHALL define a container healthcheck that calls `GET /health` on the bridge.

#### Scenario: Container healthcheck configured
- **WHEN** the Docker image is built
- **THEN** the container healthcheck invokes the bridge `/health` endpoint

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
