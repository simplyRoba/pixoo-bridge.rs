## Why

We need a stable, typed foundation for talking to Pixoo devices because the current HTTP API is inconsistent (always POST, odd content-types, variable payloads) and hard to use safely across the codebase. Establishing a single client now keeps future features from re-implementing brittle request/response handling.

## What Changes

- Introduce a generic Pixoo HTTP client that posts command payloads to a configured device IP using JSON bodies and parses JSON responses even when the content-type is incorrect.
- Standardize command dispatch via a command enum plus argument maps so variable request fields are supported while keeping the command type explicit.
- Validate `error_code` on every response and only return the remaining response fields when the call succeeds.

## Capabilities

### New Capabilities
- `pixoo-http-client`: Define how the bridge constructs POST requests for Pixoo commands, parses response payloads despite inconsistent content-types, and returns structured data after `error_code` validation.

### Modified Capabilities

## Impact

- New client module(s) to encapsulate Pixoo command serialization, HTTP request execution, and response parsing/validation.
- Updates to public bridge APIs that need to issue Pixoo commands to route through the client.
- No new external services; relies on existing HTTP/serde stack.
