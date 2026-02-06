## 1. Setup and Preparation

- [ ] 1.1 Review existing `/manage` endpoint patterns and structures
- [ ] 1.2 Identify common utilities and helpers used in current endpoints
- [ ] 1.3 Set up test infrastructure for new display endpoints

## 2. Implementation of Display Control Endpoints

- [ ] 2.1 Implement `/manage/display/on/{action}` endpoint for toggling display power
- [ ] 2.2 Implement `/manage/display/brightness/{value}` endpoint for setting brightness
- [ ] 2.3 Implement `/manage/display/rotation/{angle}` endpoint for display rotation
- [ ] 2.4 Implement `/manage/display/mirror/{action}` endpoint for mirror mode control
- [ ] 2.5 Implement `/manage/display/highlight/{action}` endpoint for highlight mode control
- [ ] 2.6 Implement `/manage/display/white-balance` endpoint for white balance adjustment

## 3. Input Validation and Error Handling

- [ ] 3.1 Add validation for display on/off action parameter
- [ ] 3.2 Add validation for brightness value (0-100 range)
- [ ] 3.3 Add validation for rotation angle (0, 90, 180, 270 degrees)
- [ ] 3.4 Add validation for mirror mode action parameter
- [ ] 3.5 Add validation for highlight mode action parameter
- [ ] 3.6 Add validation for white balance RGB values (0-100 range)
- [ ] 3.7 Implement consistent error responses with HTTP 400 for invalid inputs

## 4. Pixoo Command Mapping

- [ ] 4.1 Create mapping from `/manage/display/on` to `Channel/OnOffScreen` command
- [ ] 4.2 Create mapping from `/manage/display/brightness` to `Channel/SetBrightness` command
- [ ] 4.3 Create mapping from `/manage/display/rotation` to `Device/SetScreenRotationAngle` command
- [ ] 4.4 Create mapping from `/manage/display/mirror` to `Device/SetMirrorMode` command
- [ ] 4.5 Create mapping from `/manage/display/highlight` to `Device/SetHighLightMode` command
- [ ] 4.6 Create mapping from `/manage/display/white-balance` to `Device/SetWhiteBalance` command

## 5. Integration with Pixoo Client

- [ ] 5.1 Integrate endpoints with existing Pixoo client
- [ ] 5.2 Apply retry/backoff mechanisms to all display control commands
- [ ] 5.3 Handle Pixoo error responses appropriately
- [ ] 5.4 Map Pixoo success/failure responses to HTTP status codes

## 6. Testing

- [ ] 6.1 Write unit tests for input validation
- [ ] 6.2 Write unit tests for Pixoo command mapping
- [ ] 6.3 Write integration tests for successful display control operations
- [ ] 6.4 Write integration tests for error scenarios
- [ ] 6.5 Test edge cases (invalid inputs, network failures, etc.)

## 7. Documentation

- [ ] 7.1 Update API documentation to include new display endpoints
- [ ] 7.2 Add examples for each display control endpoint
- [ ] 7.3 Document error responses and status codes
- [ ] 7.4 Update README if user-facing changes are significant

## 8. Quality Assurance

- [ ] 8.1 Run `cargo fmt` to ensure code formatting
- [ ] 8.2 Run `cargo clippy` to catch potential issues
- [ ] 8.3 Run `cargo test` to verify all tests pass
- [ ] 8.4 Review code for consistency with existing patterns
- [ ] 8.5 Verify no breaking changes to existing functionality
