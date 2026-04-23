# Contributing

## Architecture

A single Rust binary that consumes the Pixoo device's proprietary HTTP protocol and re-exposes a clean REST API. Built on [axum](https://github.com/tokio-rs/axum) + [tokio](https://tokio.rs/), with [reqwest](https://github.com/seanmonstar/reqwest) as the outgoing HTTP client to the device. There is no database, no frontend, and no persistent state -- the bridge is a stateless proxy.

### Project layout

```
src/
  main.rs            # entry point, server bootstrap, graceful shutdown
  config.rs          # environment-based configuration
  routes/            # axum route handlers grouped by domain (draw, manage, tools, system)
  pixoo/             # Pixoo device client and command serialization
  middleware/        # request-id, logging
tests/               # integration tests using httpmock
```

## Dev setup

### Prerequisites

- Rust stable toolchain (latest)
- A Pixoo device on the network, or willingness to mock it

A devcontainer config is included (`.devcontainer/`) with Rust, Node, Docker-in-Docker, and Starship pre-installed. Open the project in VS Code or GitHub Codespaces and the environment is ready.

### Running locally

```bash
PIXOO_BASE_URL=http://<your-pixoo-ip> cargo run
```

The bridge binds to port `4000` by default. Override with `PIXOO_BRIDGE_PORT`.

### Testing

Run the full test suite:

```bash
cargo test
```

Tests use [httpmock](https://github.com/alexliesenfeld/httpmock) to simulate the Pixoo device -- no physical hardware required.

### Linting and formatting

```bash
cargo fmt
cargo clippy
```

Clippy runs with `pedantic = deny` (configured in `Cargo.toml`). Fix all warnings before committing.

### Pre-commit checklist

```bash
cargo fmt && cargo clippy && cargo test
```

## Commits

Follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/). Examples:

- `feat: add /draw/gif endpoint`
- `fix: return 400 on missing color field`
- `fix(deps): bump tokio from 1.51 to 1.52`
- `fix(ci): update checkout action to v4`

## Releases

Releases are managed by [release-please](https://github.com/googleapis/release-please). Merging to `main` automatically creates or updates a release PR. Once that PR is merged, the pipeline compiles binaries for `linux/amd64` and `linux/arm64`, uploads them as GitHub release assets, and publishes a multi-arch Docker image to `ghcr.io`.

## Docker

Build the production image locally (requires pre-built binaries in `release-artifacts/`):

```bash
docker build -t pixoo-bridge .
```

For local testing, the `docker-compose.yaml` pulls the latest published image:

```bash
docker compose up -d
```
