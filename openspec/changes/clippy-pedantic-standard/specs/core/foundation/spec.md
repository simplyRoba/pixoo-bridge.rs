# Delta Spec: Core Foundation - Clippy Pedantic

## Changes to core/foundation

### Lint Configuration

The project enforces clippy pedantic lints via `Cargo.toml`:

```toml
[lints.clippy]
pedantic = { level = "warn", priority = -1 }
missing_errors_doc = "allow"
missing_panics_doc = "allow"
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
