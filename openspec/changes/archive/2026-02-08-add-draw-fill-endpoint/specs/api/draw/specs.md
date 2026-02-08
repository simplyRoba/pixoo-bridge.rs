## ADDED Requirements

### Requirement: Draw fill endpoint exposes single-color automation
The system SHALL provide a `POST /draw/fill` API that accepts a JSON body containing `red`, `green`, and `blue` integer values between 0 and 255 inclusive. The handler SHALL return `200 OK` with an empty body when the Pixoo device successfully accepts the animation and appropriate error responses when the request fails validation or the device reports an error.

#### Scenario: Valid color fills display
- **WHEN** a client posts `{"red": 32, "green": 128, "blue": 16}` to `/draw/fill`
- **THEN** the server responds with `200 OK` and an empty body, and the Pixoo device receives the automation that paints every pixel in the requested RGB color.

### Requirement: Pixoo command flow for draw fill
The draw handler SHALL internally fetch a fresh animation ID via `Draw/GetHttpGifId` before issuing a single-frame automation with `Draw/SendHttpGif`. The automation arguments SHALL set `PicNum=1`, `PicOffset=0`, `PicWidth=64`, and include the previously fetched `PicID`.

#### Scenario: Pixoo commands are sequenced correctly
- **WHEN** the handler receives a valid fill request
- **THEN** it first calls `Draw/GetHttpGifId`, then calls `Draw/SendHttpGif` with the metadata described above, and the response is `error_code=0` for both commands.

### Requirement: Reusable pixel data helper
The system SHALL provide a helper that converts a 64×64 pixel buffer (left-to-right, top-to-bottom) into the Base64-encoded `PicData` payload required by Pixoo. This helper SHALL be usable by any future draw endpoints and SHALL emit bytes in the order `[red, green, blue]` per pixel without padding.

#### Scenario: Base64 helper produces expected payload
- **WHEN** the helper receives a uniform 64×64 buffer where every pixel is `(255, 0, 128)`
- **THEN** the returned string is the Base64 encoding (without padding) of the 64×64×3 byte sequence representing the requested color in row-major order.

### Requirement: Pixoo command modeling and client reuse
The Pixoo command enumeration and client SHALL include the draw automation commands (`Draw/SendHttpGif`, `Draw/GetHttpGifId`, `Draw/ResetHttpGifId`) so that the existing retry/backoff and response-handling logic can be reused by the new endpoint.

#### Scenario: Draw commands reuse Pixoo client behavior
- **WHEN** the draw route triggers Pixoo traffic
- **THEN** the requests go through `PixooClient::send_command`, benefiting from the same logging, retry/backoff, and error normalization that other commands already use.
