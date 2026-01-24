# Project Context

## Purpose
pixoo-bridge.rs consumes the Pixoo LED matrix's proprietary protocol and re‑exposes its own HTTP API so orchestration systems (Home Assistant, automation platforms, etc.) can control the matrix easily without touching the vendor's ugly API.

## Tech Stack
- Rust (stable toolchain via `cargo`)
- Native networking (HTTP/UDP) plus any minimal crates needed to talk to the Pixoo device; no frontend/runtime dependencies.
- Standard formatter/workflow via `rustfmt`/`cargo clippy`.

## Project Conventions

### Code Style
Follow Rust idioms and tooling: prioritize readability, explicit error handling with `Result`, `snake_case` for functions/variables, and `UpperCamelCase` for public types. Keep `Cargo.toml` tidy and run `cargo fmt` before committing.

### Architecture Patterns
Standalone Rust bridge service: single crate providing an HTTP server that translates incoming REST/JSON commands into the device-specific protocol. Keep the HTTP layer thin, push all Pixoo-specific framing to dedicated modules/traits, and isolate transport retries so the bridge can stay responsive and testable.

### Testing Strategy
Run `cargo test` for unit/regression coverage. For device-focused behavior, rely on mocks or fixtures that emulate the Pixoo API; keep actual hardware calls at the edges of the crate.

### Git Workflow
Trunk-based development on `main`. Create short-lived feature branches named after the task. Commits should be focused, human-readable, and mention the why; run `cargo fmt && cargo test` locally before committing. Pushing is never allowed; only local commits are recorded until someone else handles upstream sync.

## Domain Context
The Pixoo LED matrix only exposes an awkward proprietary control API. This project acts as a bridge (not a library) so downstream automation systems can issue simple HTTP commands (`display_frame`, `set_gradient`, etc.) while the service handles packet formatting, command sequencing, retries, and acknowledgement quirks.

## Important Constraints
Plans must account for the Pixoo API's quirks (unfriendly endpoints, inconsistent acknowledgement) and the limited runtime environment of the device (no heavy async frameworks). Network reliability can vary, so keep retries/simple backoff in mind.

## External Dependencies
- Pixoo's proprietary HTTP/UDP API (over the local network) is the primary external integration; minimize assumptions about its stability and avoid leaking protocol details.
