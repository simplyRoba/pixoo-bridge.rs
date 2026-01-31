## Context

`ci.yml` currently runs checkout, toolchain installs, cache setup, fmt/clippy, tests, and release builds inside a single job, while `publish-release.yml` repeats the same compilation logic inside Docker. Each step is serial, failures obscure downstream behavior, and the redundant compilation keeps caches alive even for simple lint/test changes. Uploading release binaries from CI duplicates effort already handled in the publish workflow and makes it harder to reason about where artifacts originate.

## Goals / Non-Goals

**Goals:**
- Separate linting and testing into dedicated jobs so failures stay scoped and run faster.
- Provide a single setup job that handles toolchain installs, target additions, and caching so downstream jobs reuse the same environment.
- Move release binary compilation entirely to `publish-release.yml`, preserve the Dockerfile’s current commands, and publish those binaries as release assets.
- Simplify the Dockerfile so it copies a prebuilt binary per `TARGETPLATFORM` instead of rebuilding inside Docker.
- Split `publish-release.yml` into scoped jobs (setup, compile, upload assets, docker build) so each stage communicates its outcome clearly.

**Non-Goals:**
- Leaving CI responsible for release binary compilation or uploads.
- Adding heavy dependencies to the runtime image beyond the current `curl`/`ca-certificates` footprint.

## Decisions

1. **Centralize setup/caching in CI.**
   - Responsibility: install `rustfmt`, `clippy`, add the x86_64 and aarch64 targets, install cross compilers, and prime the cache via `actions/cache`. The lint and test jobs `need` this job to benefit from the same environment.
   - Alternative: inline setup per job (repetition). That approach was rejected to keep tracing dependencies simple.

2. **Keep lint/test jobs short.**
   - Design: lint job runs `cargo fmt -- --check` and `cargo clippy -- -D warnings`; test job runs `cargo test`. Both restore the same cache key defined in setup.
   - Alternative: combine lint/test; rejected because failure contexts blur and jobs become slow to rerun.

3. **Compile release binaries in the publish workflow.**
   - Steps: reinstall cross compilers, add both targets, run the Dockerfile-aligned `cargo build --release --target` commands, and upload the resulting binaries as release assets before building the Docker image.
   - Alternative: continue building in Docker; rejected because it duplicates compile time and forces cross compilers into each job.

4. **Copy prebuilt binaries inside Docker.**
   - Dropbox: Docker just copies `/target/<triple>/release/pixoo-bridge` into `/usr/local/bin` depending on `TARGETPLATFORM` and removes unused artifacts afterward.
   - Alternative: keep multi-stage builds; rejected because it renders the Docker build slower while producing binaries that already exist elsewhere.

5. **Publish workflow job split.**
   - Structure `publish-release.yml` into multiple jobs: a setup job for compilers/deps, a compile job for both binaries, an upload job that attaches assets to the release, and a final job that builds/pushes the Docker image using the prebuilt files.
   - Alternative: keep a single job; rejected because a failure in upload currently aborts the Docker build without clear attribution.

## Risks / Trade-offs

- [Publish workflow duration] → Installing compilers and running two `cargo build` commands increases runtime; mitigate by reusing the same commands the Dockerfile already executes, so people already know what to expect.
- [Docker context size] → Copying `target` increases context; mitigate by deleting unused binaries inside the Dockerfile after placing the correct binary.
- [Asset mismatch] → Release assets must match the Docker image; ensure Docker copies those exact binaries and document the requirement so future changes don’t diverge.
- [Cache drift] → A cache key tied to `Cargo.lock` and runner OS prevents frequent cache misses for lint/test jobs.

## Migration Plan

1. Keep `.github/workflows/ci.yml` focused on setup, lint, and test jobs with the shared cache key.
2. Split `.github/workflows/publish-release.yml` into setup/compile/upload/docker build jobs and ensure each job documents its dependency chain.
3. Simplify `Dockerfile` to copy the correct prebuilt binary based on `TARGETPLATFORM` and clean up unused artifacts.
4. Note the new release asset process in the README and verify the release workflow publishes both binaries.

## Open Questions

- Should we add a manual workflow for spot-checking binaries outside publish-release?
- Does CI need any placeholder release artifact for local dev, or is publish-release alone sufficient for downstream consumers?
