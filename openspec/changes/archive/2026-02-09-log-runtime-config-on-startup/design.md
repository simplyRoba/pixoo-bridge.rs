## Context

The bridge already emits a structured info log during startup that lists the resolved listener address, Pixoo base URL, and binary version. The animation speed factor and maximum image size are determined by environment overrides but are not surfaced anywhere observers can easily capture. Without these values in the log, operators must rely on inspecting the configuration source or reproducing the deployment to understand why animations behave differently across clusters.

## Goals / Non-Goals

**Goals:**
- Expand the startup log entry so it captures `animation_speed_factor` and `max_image_size` alongside the existing configuration metadata.
- Keep the change localized to the logging path that already runs before incoming traffic is accepted.
- Ensure the logging requirement in `core/logging` reflects the behavior produced by the code.

**Non-Goals:**
- No new operator-facing APIs or runtime configuration changes.
- No additional dependencies or formatting libraries.

## Decisions
- **Inline the additional fields in the existing log entry:** Instead of introducing a separate log line, extend the info log already emitted after `AppState` is constructed so the values stay grouped with the other runtime configuration details.
- **Log `animation_speed_factor` as a floating-point value and `max_image_size` as an integer (bytes):** This keeps the log structured and easily filterable while matching the types already used in the configuration and downstream code that applies those limits.
- **Use the resolved values from `AppState` rather than re-reading environment variables:** The config loader already applies validation, so logging the values stored on `AppState` guarantees observers see the same values the runtime uses.

## Risks / Trade-offs
- [Minimal exposure] → The new log fields expose additional configuration details, but they are not secrets and are already derivable from the environment variables that operators control.
- [Log verbosity] → Adding two extra fields slightly increases log size; however, this is a single info event emitted at startup, so the impact on storage is negligible.

## Migration Plan
- No rollout steps are required; once deployed, the bridge will emit the additional fields during startup and the spec will describe the new behavior.

## Open Questions
- None.
