# Proposal: Clippy Pedantic as Project Standard

## Problem Statement

The codebase lacks consistent enforcement of Rust idioms and best practices. Code style varies across modules, and potential issues that clippy's pedantic lints would catch go unnoticed.

## Proposed Solution

Enable clippy pedantic lints project-wide via `Cargo.toml` configuration, making it the standard for all code in the repository. This ensures consistent code quality and catches common issues early.

## Scope

- Add `[lints.clippy]` configuration to `Cargo.toml`
- Fix all existing pedantic lint violations
- Selectively allow lints that are too noisy for this project

## Success Criteria

- `cargo clippy --all-targets -- -D warnings` passes with pedantic enabled
- All tests continue to pass
- Code follows consistent Rust idioms
