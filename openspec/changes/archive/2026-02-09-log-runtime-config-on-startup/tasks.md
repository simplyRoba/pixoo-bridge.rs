## 1. Startup logging

- [x] 1.1 Extend the structured startup info log to include the resolved `animation_speed_factor` and `max_image_size` values alongside the existing fields.
- [x] 1.2 Ensure the log entry uses the validated values stored on `AppState` so that what gets logged is what the runtime applies.

## 2. Specs and documentation

- [x] 2.1 Update `openspec/specs/core/logging/specs.md` so the startup logging requirement and scenario describe the new fields.
- [x] 2.2 Refresh the change-specific spec delta (`openspec/changes/log-runtime-config-on-startup/`) so the artifacts describe the updated behavior.

## 3. Validation

- [x] 3.1 Run `cargo fmt`, `cargo clippy`, and `cargo test`.
