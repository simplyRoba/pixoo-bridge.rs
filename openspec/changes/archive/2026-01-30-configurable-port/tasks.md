## 1. Listener configuration

- [x] 1.1 Add a helper that reads `PIXOO_BRIDGE_PORT`, defaults to `4000`, parses as a `u16`, and warns/falls back when the value is missing or out of the valid `1024..=65535` range.
- [x] 1.2 Use the resolved port when constructing the `SocketAddr` bound by Axum and log the configured port alongside the existing startup context.

## 2. Documentation & deployment

- [x] 2.1 Update the `Dockerfile` to expose the default port (4000) and document that `PIXOO_BRIDGE_PORT` controls the listener so container mappings stay in sync.
- [x] 2.2 Describe the new configuration knob, default port, and expected behavior in `README.md` (and CHANGELOG if necessary) so operators know how to override the listener.

## 3. Testing & verification

- [x] 3.1 Add/extend unit tests around the helper so valid ports, missing values, and invalid inputs (non-numeric or out-of-range) all behave as specified.
- [x] 3.2 Run `cargo fmt`, `cargo clippy`, and `cargo test` to prove the codebase stays healthy after the change.
