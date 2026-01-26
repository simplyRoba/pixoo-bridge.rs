## Git workflow
- Always branch from `main`; AI may create branches but must never merge or push to `main`.
- Keep branches short-lived, focused on a single change, and clearly named (e.g., `feat/pixoo-retry` rather than `wip`).
- Commits should follow Conventional Commits (`feat`, `fix`, etc.) per https://www.conventionalcommits.org/en/v1.0.0/#specification.

## Review expectations
- Treat every change as pending until a human explicitly reviews it; nothing merges without that approval.
- Before requesting review, run `cargo fmt`, `cargo clippy`, and `cargo test` so artifacts and implementation stay in sync.

## Tooling constraints
- Rust work stays on the latest stable toolchain via `rustup`; do not depend on nightly-only features or pin a custom channel in `AGENTS.md`.
- If the CLI is used, prefer the bundled OpenSpec commands (`openspec status`, `openspec instructions`, etc.) that read from the current repo structure.
