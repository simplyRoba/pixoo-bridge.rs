## 1. Validation and action enums

- [x] 1.1 Define shared request structs that derive `Deserialize`, `Validate`, and any existing schema traits while encoding the numeric/text rules from the request-validation spec so handlers get normalized data.
- [x] 1.2 Introduce enumerable action parameters (with `rename_all = "lowercase"`) and reuse them across relevant tool routes per the api/common spec to reject unsupported values during extraction.
- [x] 1.3 Update `src/routes/tools.rs` to use the new extractors/enums, keep the current JSON responses, and stop performing inline range/action checks.
- [x] 1.4 Bridge validation failures into the shared HTTP error response (error+details) described in the design so malformed payloads return consistent 400-series responses.

## 2. Tests, documentation, and release hygiene

- [x] 2.1 Extend the tool route tests to cover the shared validation paths (valid inputs, out-of-range values, unsupported actions) while keeping payloads/test assertions aligned with the specs.
- [x] 2.2 Run `cargo fmt`, `cargo clippy`, and `cargo test`, and ensure any validator-derived dependencies are wired into the Docker-friendly toolchain before archiving.
