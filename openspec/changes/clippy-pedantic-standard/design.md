# Design: Clippy Pedantic as Project Standard

## Configuration

Add clippy pedantic lints to `Cargo.toml`:

```toml
[lints.clippy]
pedantic = { level = "warn", priority = -1 }
# Documentation lints - too noisy for internal tool
missing_errors_doc = "allow"
missing_panics_doc = "allow"
# Style preferences
must_use_candidate = "allow"
module_name_repetitions = "allow"
```

## Allowed Exceptions

- `missing_errors_doc` / `missing_panics_doc`: Documentation requirements are too verbose for an internal tool
- `must_use_candidate`: Too many false positives for builder-style APIs
- `module_name_repetitions`: Acceptable for clarity (e.g., `PixooError` in `pixoo` module)

## Code Changes Required

1. **Safe integer casts**: Replace `as` casts with `try_from().unwrap_or()` or `try_from().ok()`
2. **Let-else patterns**: Use `let ... else` instead of `match` for early returns
3. **Pass by reference**: Change owned parameters to references where appropriate
4. **Match arm consolidation**: Merge identical match arms
5. **Explicit variants**: Replace wildcard patterns with explicit enum variants
6. **Idiomatic combinators**: Use `map_or_else` instead of `map().unwrap_or_else()`
