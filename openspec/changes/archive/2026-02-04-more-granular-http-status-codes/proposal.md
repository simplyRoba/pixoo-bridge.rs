## Why

Pixoo device failures currently collapse to HTTP 503, so callers cannot distinguish unreachable devices, unrecoverable device health, or timeouts. Aligning bridge error responses with more precise HTTP semantics will help automation layers react appropriately instead of treating every failure as generic service unavailability.

## What Changes

- Define a reusable mapping from Pixoo client outcomes (connection refusal, command failure, timeouts, etc.) to the HTTP status codes clients expect (502 for unreachable devices, 503 for unhealthy responses, 504 for request timeouts) and document how each case is selected.
- Update the HTTP routing layer so existing endpoints such as `/health`, `/manage/*`, and `/reboot` consult the new mapping when translating Pixoo client errors into bridge responses, ensuring the payload still contains a human-readable error body.

## Capabilities

### New Capabilities
- `api/common`: Add requirements describing how Pixoo client error conditions translate into HTTP responses (502/503/504) with structured payloads, extending the existing common API behavior.

### Modified Capabilities
- *none*

## Impact

- HTTP handlers for `/health`, `/manage/*`, `/reboot`, and any other Pixoo-exposed routes will consult the new mapping before choosing their response code.
- The Pixoo client’s error surface and any middleware that formats bridge errors must expose enough detail (error kind, timeout flag, unreachable flag) for the mapping to pick an accurate status.
- Operators consuming the bridge’s HTTP API may see changed status codes, so the release should be communicated in release notes and health monitoring expectations.
