# Delta Spec: Core Foundation - Clippy Pedantic

## Changes to core/foundation

### Lint Configuration

The project enforces clippy pedantic lints via `Cargo.toml`:

```toml
[lints.clippy]
pedantic = { level = "warn", priority = -1 }
must_use_candidate = "allow"
module_name_repetitions = "allow"
```

### Code Standards

All code must pass `cargo clippy --all-targets -- -D warnings` with pedantic lints enabled.

Required patterns:
- Use `let ... else` for early returns instead of `match`
- Use safe integer conversions (`try_from`) instead of `as` casts
- Pass by reference when ownership is not needed
- Use explicit enum variants instead of wildcards in exhaustive matches
- Consolidate identical match arms

### Documentation Standards

Public functions returning `Result` must include an `# Errors` section documenting when each error variant is returned:

```rust
/// Description of the function.
///
/// # Errors
///
/// Returns [`ErrorType::Variant`] if condition occurs.
pub fn example() -> Result<T, ErrorType> {
```
