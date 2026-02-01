## Why

The existing CI workflow runs lint, tests, builds, and artifact uploads in a single linear job, which makes failures harder to diagnose, stretches runtime by serializing independent steps, and buries the cache setup in a long script. Clarifying responsibilities and introducing parallelism will keep the wasm bridge fast, make breakages obvious, and allow later extensions without additional sequential cruft.

## What Changes

- Split `.github/workflows/ci.yml` into focused jobs (toolchain setup, lint, and tests) instead of a single monolith so the workflow finishes faster and failures stay scoped.
- Update `.github/workflows/publish-release.yml` to build the release binaries using the same commands the Dockerfile used, split the workflow into scoped jobs (setup, compile, upload assets, docker build), upload the binaries as release assets, and pass the prebuilt artifacts into the Docker build so we only compile once.
- Simplify the Dockerfile to just copy the precompiled binaries for the target platform, and document in the README that release binaries now come from the publish workflow.

## Capabilities

### New Capabilities
- _None_ (all requirements extend existing workflow guarantees)

### Modified Capabilities
- `core/github-workflows`: Expand the workflow requirements to document the new CI job layout, cached setup, publish-release binary builds (matching the Dockerfile), and release asset uploads so automation stays clear about where each artifact originates.

## Impact

- `.github/workflows/ci.yml` will be reorganized into separate setup, lint, and test jobs, removing the release-build/upload responsibilities.
- `.github/workflows/publish-release.yml` will grow a compile stage that runs the same commands as the Dockerfile, uploads the resulting binaries to the GitHub release, and hands those binaries to the Docker build.
- `Dockerfile` will drop its builder stages and simply copy the prebuilt binaries for the active `TARGETPLATFORM`, keeping the runtime image small.
