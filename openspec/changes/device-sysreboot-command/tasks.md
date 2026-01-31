## 1. System routing

- [ ] 1.1 Create a `routes/system` module (or similar) that exports a `mount_system_routes` helper and move the `/health` handler into it.
- [ ] 1.2 Update `main.rs` to mount the new system routes module instead of inlining the handlers, ensuring shared middleware applies to `/health` and `/reboot`.

## 2. Pixoo command support

- [ ] 2.1 Extend the Pixoo command router/types to include a `DeviceSysReboot` variant and reuse the existing framing/retry logic.
- [ ] 2.2 Wire the `/reboot` handler in `routes/system` to call the new capability, translating failures into HTTP 503 and successes into 204.

## 3. Documentation and verification

- [ ] 3.1 Update the README/CHANGELOG to mention the `/reboot` endpoint and note the `/health` handler’s new home.
- [ ] 3.2 Add or update unit/integration tests covering the `/health` route and the new `/reboot` handler.
