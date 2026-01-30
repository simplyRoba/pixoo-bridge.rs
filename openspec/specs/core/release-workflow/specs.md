# core/release-workflow Capability

## Requirements

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
