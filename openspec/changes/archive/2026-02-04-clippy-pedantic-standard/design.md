# Design: Clippy Pedantic as Project Standard

## Context

The project uses Rust with axum for the HTTP bridge. Clippy provides linting but pedantic lints are not enabled, allowing inconsistent code patterns.

## Goals / Non-Goals

**Goals:**
- Enforce consistent Rust idioms via clippy pedantic
- Ensure all public API functions have error documentation
- Pass `cargo clippy --all-targets -- -D warnings`

**Non-Goals:**
- Enforce documentation lints that are too verbose (`must_use_candidate`)
- Rename types to avoid module repetition (`module_name_repetitions`)

## Decisions

### Decision 1: Configure via Cargo.toml

Use `[lints.clippy]` in `Cargo.toml` rather than `#![warn(clippy::pedantic)]` in code. This centralizes configuration and allows selective overrides.

```toml
[lints.clippy]
pedantic = { level = "warn", priority = -1 }
must_use_candidate = "allow"
module_name_repetitions = "allow"
```

### Decision 2: Use safe integer conversions

Replace `as` casts with `try_from().unwrap_or()` for safety:
```rust
let delay = self.backoff * u32::try_from(attempt).unwrap_or(u32::MAX);
```

### Decision 3: Prefer let-else for early returns

Use `let ... else` pattern instead of `match` for Option unwrapping with early return:
```rust
let Some(client) = state.pixoo_client.clone() else {
    return error_response();
};
```
