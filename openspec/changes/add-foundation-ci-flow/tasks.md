## 1. Implementation
- [ ] 1.1 Create the Rust crate layout (`Cargo.toml`, `src/main.rs`) with a minimal HTTP bridge entry point.
- [ ] 1.2 Declare the elemental async/HTTP/serialization dependencies that the bridge will rely on.
- [ ] 1.3 Add scaffolding modules (e.g., `transport`, `pixoo_proto`) or placeholders if needed to keep `cargo check` passing.

## 2. CI and Release
- [ ] 2.1 Write `.github/workflows/ci.yml` that installs Rust, checks formatting, runs `cargo test`, and builds release artifacts on PRs and pushes.
- [ ] 2.2 Write `.github/workflows/release-please.yml` that runs release-please to create release branches and GitHub releases on merge to main.
- [ ] 2.3 Write `.github/workflows/publish-release.yml` that publishes the Docker image to GHCR after a GitHub release is published.

## 3. Validation
- [ ] 3.1 Run `cargo fmt`/`cargo test` locally to ensure the new crate builds.
- [ ] 3.2 Use `openspec validate add-foundation-ci-flow --strict --no-interactive` once the rest of the change lands.
