## 1. Test Coverage

- [x] 1.1 Add unit tests for text payload validation (id, width, font, speed, alignment, color, text length)
- [x] 1.2 Add handler tests for `/draw/text` success and validation failure responses
- [x] 1.3 Add handler tests for `/draw/text/clear` success and Pixoo error mapping

## 2. Core Implementation

- [x] 2.1 Add Pixoo command models for `Draw/SendHttpText` and `Draw/ClearHttpText`
- [x] 2.2 Implement typed text request payload struct and validation helpers
- [x] 2.3 Implement `/draw/text` handler wiring and command dispatch via existing Pixoo client
- [x] 2.4 Implement `/draw/text/clear` handler wiring and command dispatch via existing Pixoo client

## 3. Documentation and Verification

- [x] 3.1 Review `README.md` and update user-facing API docs if needed
- [x] 3.2 Run `cargo fmt`, `cargo clippy`, and `cargo test`
