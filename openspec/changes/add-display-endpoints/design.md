## Context

The Pixoo bridge is a Rust-based HTTP bridge that communicates with Pixoo display devices. This design document outlines the implementation approach for adding display control endpoints under the `/manage/display/*` path prefix. These endpoints will provide a user-friendly interface for managing display settings including power state, brightness, rotation, mirroring, highlight mode, and white balance.

The current implementation already has a `/manage` endpoint group that handles some display-related settings. The new `/manage/display/*` endpoints will extend this group with a dedicated, focused API surface specifically for display control.

## Goals / Non-Goals

**Goals:**
- Provide a clear, dedicated API surface for display control operations
- Maintain consistency with existing endpoint patterns and error handling
- Reuse existing Pixoo client and retry/backoff mechanisms
- Ensure proper input validation before sending commands to Pixoo
- Keep the implementation simple and maintainable

**Non-Goals:**
- Redesign or refactor existing `/manage` endpoints
- Add new display features beyond what Pixoo supports
- Implement complex display management workflows
- Add authentication or authorization mechanisms (if not already present)

## Decisions

### Decision 1: Endpoint Structure and Naming
**Choice**: Use RESTful path parameters for simple on/off and value-based controls, and JSON body for complex configurations.

**Rationale**: 
- Path parameters like `/manage/display/on/{action}` and `/manage/display/brightness/{value}` are intuitive and easy to use
- JSON body for `/manage/display/white-balance` allows for structured data with multiple fields
- This approach is consistent with existing endpoints in the `/manage` group

**Alternatives Considered**:
- Query parameters: Less clean for required values, harder to document
- JSON body for all endpoints: More verbose for simple operations like on/off

### Decision 2: Error Handling Strategy
**Choice**: Validate inputs before sending commands to Pixoo and return HTTP 400 for invalid requests.

**Rationale**:
- Prevents invalid commands from reaching the Pixoo device
- Provides immediate feedback to clients
- Reduces unnecessary network traffic and retries
- Consistent with existing error handling patterns

**Alternatives Considered**:
- Send commands and let Pixoo reject invalid requests: Would waste network resources and add latency
- Use HTTP 422 (Unprocessable Entity): Less commonly understood than 400

### Decision 3: Pixoo Command Mapping
**Choice**: Create a mapping layer that translates display control requests to Pixoo-specific commands.

**Rationale**:
- Abstracts Pixoo's proprietary API details from the HTTP interface
- Makes it easier to change Pixoo command formats in the future
- Centralizes the knowledge of Pixoo's command structure
- Follows the existing pattern used in `/manage` endpoints

**Alternatives Considered**:
- Direct mapping in each handler: Would duplicate Pixoo command knowledge
- Configuration-driven mapping: Overkill for this use case

**Alternatives Considered**:
- Direct mapping in each handler: Would duplicate Pixoo command knowledge
- Configuration-driven mapping: Overkill for this use case

### Decision 4: Input Validation
**Choice**: Implement strict validation for all inputs with clear error messages.

**Rationale**:
- Ensures only valid commands reach the Pixoo device
- Provides helpful feedback to API consumers
- Prevents potential device errors or unexpected behavior
- Consistent with existing validation patterns

**Alternatives Considered**:
- Minimal validation: Would risk sending invalid commands to Pixoo
- Silent failure: Poor user experience

### Decision 5: Retry and Backoff Strategy
**Choice**: Reuse existing retry/backoff mechanisms from the Pixoo client.

**Rationale**:
- Consistent behavior across all endpoints
- Proven to work with Pixoo's flaky API
- No need to reinvent existing solutions
- Reduces code duplication

**Alternatives Considered**:
- Custom retry logic: Would duplicate existing functionality
- No retries: Would make the system more fragile

## Risks / Trade-offs

### Risk: Pixoo API Inconsistencies
**Impact**: Pixoo's API may behave unexpectedly or change without notice.
**Mitigation**: 
- Use existing retry/backoff mechanisms
- Implement comprehensive error handling
- Monitor endpoint success/failure rates
- Document known Pixoo API quirks

### Risk: Input Validation Overhead
**Impact**: Validation adds processing time before commands are sent.
**Mitigation**:
- Keep validation logic simple and efficient
- Validate only what's necessary
- Cache validation patterns where possible

### Risk: API Design Changes
**Impact**: Future requirements may necessitate API changes.
**Mitigation**:
- Design endpoints to be extensible
- Use versioning if major changes are needed
- Deprecate rather than remove endpoints

### Risk: Performance Under Load
**Impact**: High volume of display control requests could impact performance.
**Mitigation**:
- Implement rate limiting if needed
- Optimize Pixoo command batching
- Monitor endpoint response times
