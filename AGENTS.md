## Personas
- Developer: Maintains the Rust bridge, ships Docker images, and integrates Pixoo controls into automation systems.
- End user: Runs the bridge to control a Pixoo matrix from e.g. Home Assistant or simple HTTP clients.

## Git workflow
- Always branch from `main`; AI may create branches but must never merge or push to `main`.
- Keep branches short-lived, focused on a single change, and clearly named (e.g., `feat/pixoo-retry` rather than `wip`).
- Commits should follow Conventional Commits (`feat`, `fix`, etc.) per https://www.conventionalcommits.org/en/v1.0.0/#specification.

## Review expectations
- Treat every change as pending until a human explicitly reviews it; nothing merges without that approval.
- Before requesting review, run `cargo fmt`, `cargo clippy`, and `cargo test` so artifacts and implementation stay in sync.

## Clarifications
- When requirements or intent are unclear, asking for information is mandatory and preferred over proceeding with assumptions.

## Tooling constraints
- Rust work stays on the latest stable toolchain via `rustup`; do not depend on nightly-only features or pin a custom channel in `AGENTS.md`.
- If the CLI is used, prefer the bundled OpenSpec commands (`openspec status`, `openspec instructions`, etc.) that read from the current repo structure.
