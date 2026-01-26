## 1. Setup

- [x] 1.1 Create a feature branch off `main` for the Pixoo HTTP client change
- [x] 1.2 Add minimal JSON/HTTP dependencies (serde/serde_json and chosen HTTP client) to `Cargo.toml`
- [x] 1.3 Define module structure for the Pixoo client (errors, command enum, request/response handling)

## 2. Core Client Implementation

- [x] 2.1 Implement command enum and payload builder that flattens additional argument fields
- [x] 2.2 Implement HTTP POST execution with `Content-Type: application/json` to the configured device endpoint
- [x] 2.3 Implement response parsing that ignores `Content-Type` and deserializes JSON from body text
- [x] 2.4 Implement `error_code` validation and response shaping that omits `error_code` on success
- [x] 2.5 Add simple retry/backoff behavior for transient network failures

## 3. Integration

- [x] 3.1 Route existing Pixoo command call sites through the new client
- [x] 3.2 Update any public API surfaces to accept command enum plus argument map where needed

## 4. Verification and Docs

- [x] 4.1 Add unit tests for payload construction, response parsing, and error handling
- [x] 4.2 Update `README.md` with the new Pixoo client usage and behavior notes
- [x] 4.3 Run `cargo fmt`, `cargo test`, and `openspec verify` (or `/opsx:verify`)
