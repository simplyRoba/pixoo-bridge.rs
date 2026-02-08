## Why

Pixoo automations currently lack a simple API for setting every pixel to the same color, which makes it hard to build reliable foundation endpoints for more complex draw operations. Adding the `/draw/fill` endpoint now lets us wield the device with predictable single-frame automations while keeping the payload-generation logic reusable for future draw capabilities.

## What Changes

- Add a new `/draw/fill` HTTP route that accepts an RGB body, validates the range, and sequences the necessary Pixoo commands (`Draw/GetHttpGifId` followed by `Draw/SendHttpGif`).
- Introduce reusable helpers for generating 64×64 `PicData` payloads (pixels → byte array → Base64) so future draw endpoints can build on the same transformation.
- Extend the Pixoo command enumerations and client wiring to cover the draw automation commands and reuse the existing retry/response handling.
- Document the automation payload contract and any assumptions about pic IDs so further draw endpoints stay consistent.

## Capabilities

### New Capabilities
- `draw-fill`: Covers the `/draw/fill` API surface, the expectations around RGB validation, the Pixoo command sequence (`GetHttpGifId` + `SendHttpGif`), and the reusable pixel-to-Base64 helper that future draw endpoints can share.

### Modified Capabilities
- None

## Impact

- Adds a new Axum route under `src/routes/` plus request/response types and validation for RGB inputs.
- Expands `src/pixoo/command.rs` and related client logic to model the draw commands and to fetch a fresh animation ID before sending frames.
- Introduces shared picture-generation helpers (likely near the Pixoo client or a new module) so each draw endpoint can emit consistent Base64 payloads. - make this a new module!
- Tests and documentation will need updates to cover the new endpoint and payload behavior.
