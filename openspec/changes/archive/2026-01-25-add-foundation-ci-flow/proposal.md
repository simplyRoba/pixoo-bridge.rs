# Change: add base Rust bridge and CI flow

## Why
The project still lacks a runnable Rust crate and release automation, which blocks delivering the Pixoo bridge and publishing updates reliably.

## What Changes
- Add a `Cargo.toml` plus `src/` layout that compiles a minimal HTTP-exposing bridge stub.
- Declare the elemental runtime/dependency crates the bridge relies on (async runtime, HTTP server, serialization, Pixoo framing helpers).
- Create three separate GitHub Actions workflows:
  - `ci.yml`: runs lint/test/build on pull requests and pushes
  - `release-please.yml`: runs release-please to create release branches and GitHub releases on merge to main
  - `publish-release.yml`: publishes the Docker image to GHCR after a GitHub release is published

## Impact
- Affected specs: `core`
- Affected code: `Cargo.toml`, `src/main.rs`, `.github/workflows/ci.yml`, `.github/workflows/release-please.yml`, `.github/workflows/publish-release.yml`
