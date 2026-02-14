## Context

The bridge already exposes draw endpoints that map thin HTTP handlers to Pixoo draw automation commands. We need to add text endpoints that send `Draw/SendHttpText` and `Draw/ClearHttpText` while keeping the HTTP layer small, reusing existing Pixoo client retry/backoff behavior, and validating payloads consistently with other draw APIs.

## Goals / Non-Goals

**Goals:**
- Add `/draw/text` and `/draw/text/clear` routes with strict validation for IDs, position, font, width, speed, color, and alignment.
- Model Pixoo text commands as typed request payloads so serialization is explicit and reusable.
- Route Pixoo command calls through the existing client to inherit error handling and backoff.

**Non-Goals:**
- Adding new font assets or rendering text server-side; Pixoo renders the text.
- Persisting text presets or scheduling animations beyond the immediate command.

## Decisions

- Reuse the draw capability and extend the existing API module with `/draw/text` and `/draw/text/clear` handlers rather than creating a new capability, keeping specs consolidated with other draw commands.
- Use ad-hoc Pixoo command argument maps for text requests until the Pixoo client introduces typed request/response models; once available, migrate to a typed `TextRequest` (or equivalent struct) in the Pixoo command modeling layer.
- Validate request payloads at the HTTP boundary using schema/validator helpers to ensure ranges (IDs, widths, speed) and enforce UTF-8 length limits before issuing Pixoo commands.
- Use the existing `PixooClient::send_command` flow for both text commands so retries/backoff/logging stay consistent with other endpoints.

## Risks / Trade-offs

- Pixoo text behavior (scrolling, alignment) varies by firmware version → Document supported fields and return clear validation errors when unsupported values are provided.
- Strict validation may reject inputs accepted by some firmware versions → Keep validation aligned with Pixoo documented constraints and expose clear error messages to guide callers.
