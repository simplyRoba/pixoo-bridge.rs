## Why

Expose Pixoo text drawing over the HTTP bridge so automation clients can render scrolling/static text without relying on the mobile app. The bridge already standardizes draw automation; adding text endpoints now closes a common gap for dashboards and alerting flows.

## What Changes

- Add `/draw/text` and `/draw/text/clear` HTTP endpoints that map to Pixoo `Draw/SendHttpText` and `Draw/ClearHttpText` commands with validation for IDs, positions, widths, colors, and scroll options.
- Extend Pixoo command modeling and request payload helpers to support text draw commands alongside existing draw automation flows.
- Document request/response behavior and error handling for invalid text payloads and Pixoo command failures.

## Capabilities

### New Capabilities

### Modified Capabilities
- `api/draw`: Add requirements for text draw and clear endpoints, input validation, and Pixoo command sequencing for text commands.

## Impact

- HTTP API surface: new `/draw/text` and `/draw/text/clear` routes.
- Pixoo command layer: new `Draw/SendHttpText` and `Draw/ClearHttpText` request modeling.
- Validation/serialization helpers for text payloads (IDs, positions, scroll direction, font, width, speed, alignment, color).
