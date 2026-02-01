## 1. Tool routing and app wiring

- [x] 1.1 Create `routes::tools` with handlers for `/tools/timer/start`, `/tools/timer/stop`, `/tools/stopwatch/{action}`, `/tools/scoreboard`, and `/tools/soundmeter/{action}` plus a `mount_tool_routes` function.
- [x] 1.2 Update `main::build_app` to mount the new tool routes alongside `mount_system_routes` while sharing `AppState`.

## 2. Pixoo command plumbing and validation

- [x] 2.1 Expand `pixoo::command::PixooCommand` with the four `Tools/Set*` variants and ensure `as_str` returns the correct command strings.
- [x] 2.2 Define request models and helpers that validate timer durations, stopwatch actions, scoreboard scores (0..999), and soundmeter actions before translating them into Pixoo payloads.
- [x] 2.3 Reuse `PixooClient::send_command` with the new payloads so retries, logging, and error parsing remain centralized.

## 3. Tests, docs, and final polish

- [x] 3.1 Add unit tests for each tool handler covering success, Pixoo failures, missing client, and invalid inputs, plus at least one integration-style test via `build_app`.
- [x] 3.2 Document the new endpoints in `README.md` (or another user-facing doc) so operators know how to call `/tools/timer`, `/tools/stopwatch`, `/tools/scoreboard`, and `/tools/soundmeter`.
- [x] 3.3 Run `cargo fmt`, `cargo clippy`, and `cargo test` to verify the implementation and guardrails before requesting review.
