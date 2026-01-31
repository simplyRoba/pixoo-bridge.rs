## 1. System routing

- [x] 1.1 Create a `routes/system` module (or similar) that exports a `mount_system_routes` helper and move the `/health` handler into it.
- [x] 1.2 Update `main.rs` to mount the new system routes module instead of inlining the handlers, ensuring shared middleware applies to `/health` and `/reboot`.

## 2. Pixoo command support

- [x] 2.1 Extend the Pixoo command router/types to include a `DeviceSysReboot` variant and reuse the existing framing/retry logic.
- [x] 2.2 Wire the `/reboot` handler in `routes/system` to call the new capability, translating failures into HTTP 503 and successes into 204.

## 3. Documentation and verification

- [x] 3.1 Update the README/CHANGELOG to mention the `/reboot` endpoint and note the `/health` handler’s new home.
- [x] 3.2 Add or update unit/integration tests covering the `/health` route and the new `/reboot` handler.
