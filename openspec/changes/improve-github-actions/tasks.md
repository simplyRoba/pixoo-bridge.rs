## 1. CI job layout

- [ ] 1.1 Split `.github/workflows/ci.yml` into dedicated `setup`, `lint`, and `test` jobs so failures map to their responsible phase.
- [ ] 1.2 Move the cross-target toolchain installation and cache priming into the `setup` job so downstream jobs reuse the same environment.
- [ ] 1.3 Keep the release builds out of CI so the workflow finishes quickly and leaves binary publication to the release job.

## 2. Release workflow & artifacts

- [ ] 2.1 Update `.github/workflows/publish-release.yml` to compile the x86_64 and aarch64 binaries using the same commands the Dockerfile expects and upload them as release assets, then split the workflow into scoped setup/compile/upload/docker build jobs.
- [ ] 2.2 Simplify `Dockerfile` so it copies the precompiled binary for `TARGETPLATFORM` instead of rebuilding the code, and ensure the unused binary is removed.

## 3. Validation

- [ ] 3.1 Run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test` locally to confirm the repository still passes the guardrails.
- [ ] 3.2 Confirm conceptually (or via GitHub) that `publish-release.yml` now builds release binaries and attaches them to the release so nothing is missing from GHCR.
