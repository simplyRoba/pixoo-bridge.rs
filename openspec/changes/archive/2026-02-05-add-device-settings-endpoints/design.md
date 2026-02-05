## Context

The bridge exposes read-only settings for time mode and temperature unit but lacks endpoints to modify them. The Pixoo device supports these via `Device/SetTime24Flag` and `Device/SetDisTempMode`. We need to expose these capabilities safely via the HTTP API.

## Goals / Non-Goals

**Goals:**
- Enable switching between 12h/24h time format.
- Enable switching between Celsius/Fahrenheit temperature units.
- Validate input values strictly (enums) before sending commands to Pixoo.
- Use existing generic command dispatch where possible to minimize code churn.

**Non-Goals:**
- Implementing other device settings (rotation, brightness, mirror) in this specific change (scoped for future work).
- Changing how settings are *read* (GET endpoints remain as-is).

## Decisions

### 1. Endpoint Structure: RESTful vs RPC
**Decision:** use REST-like resource paths `POST /manage/time/mode/{mode}` and `POST /manage/weather/temperature-unit/{unit}`.
**Rationale:** This aligns with the existing `/manage/` hierarchy. Using path parameters allows strictly typed matching (e.g. only matching `/12h` or `/24h`) at the routing layer if the framework supports it, or simple parsing within the handler. It avoids the need for a JSON body for simple scalar toggles. `POST` is used as these are state-changing operations that map to imperative commands on the device.

### 2. Command Dispatch
**Decision:** Reuse the existing `post_command` or equivalent generic mechanism in `pixoo-client`.
**Rationale:** These commands (`Device/SetTime24Flag`, `Device/SetDisTempMode`) fit the standard pattern of "send JSON, get generic ack". No complex response parsing is needed, so specialized client methods are unnecessary overhead.

### 3. Error Handling
**Decision:** Return standard HTTP 400 for invalid path parameters (if manually parsed) and HTTP 503 if Pixoo fails.
**Rationale:** Keeps API behavior consistent with other endpoints.

## Risks / Trade-offs

- **Risk:** Pixoo generic ack might not guarantee the setting was actually applied instantly.
  - **Mitigation:** Document that this is fire-and-forget from the bridge's perspective, though we check for the device's "success" response code. Clients can verify by polling `GET /manage/settings`.

