# api/draw Capability

## Purpose
Provide a focused draw capability that lets clients fill the Pixoo panel with a single color using a thin HTTP surface. This capability serves as the foundation for future draw endpoints by enforcing consistent validation, Pixoo command sequencing, and shared payload helpers.

## Requirements

### Requirement: Draw fill endpoint exposes single-color automation
The system SHALL provide a `POST /draw/fill` API that accepts a JSON body containing `red`, `green`, and `blue` integer values between 0 and 255 inclusive. The handler SHALL return `200 OK` with an empty body when the Pixoo device successfully accepts the animation and appropriate error responses when the request fails validation or the device reports an error.

#### Scenario: Valid color fills display
- **WHEN** a client posts `{"red": 32, "green": 128, "blue": 16}` to `/draw/fill`
- **THEN** the server responds with `200 OK` and an empty body, and the Pixoo device receives the automation that paints every pixel in the requested RGB color.

### Requirement: Pixoo command flow for draw fill
The draw handler SHALL internally fetch a fresh animation ID via `Draw/GetHttpGifId` before issuing a single-frame automation with `Draw/SendHttpGif`. The automation arguments SHALL set `PicNum=1`, `PicOffset=0`, `PicWidth=64`, and include the previously fetched `PicId`. The GIF ID retrieval and frame-sending logic SHALL be implemented as shared helpers reusable by both `/draw/fill` and `/draw/upload` so that command sequencing and error handling remain consistent across draw endpoints.

#### Scenario: Pixoo commands are sequenced correctly
- **WHEN** the handler receives a valid fill request
- **THEN** it first calls `Draw/GetHttpGifId`, then calls `Draw/SendHttpGif` with the metadata described above, and the response is `error_code=0` for both commands.

#### Scenario: Upload endpoint reuses the same Pixoo command helpers
- **WHEN** the upload handler processes a valid image
- **THEN** it uses the same shared GIF ID retrieval and frame-sending helpers as `/draw/fill`, benefiting from the same error handling and response parsing.

### Requirement: Draw upload endpoint accepts image files via multipart upload
The system SHALL provide a `POST /draw/upload` API that accepts `multipart/form-data` with a field named `file` containing a JPEG, PNG, static WebP, animated WebP, or GIF image. The handler SHALL decode the image, resize it to 64×64 pixels, and send the resulting frames to the Pixoo device. On success the handler SHALL return `200 OK` with an empty body.

#### Scenario: Static JPEG upload displays on Pixoo
- **WHEN** a client posts a valid JPEG file to `/draw/upload` as multipart form data with field name `file`
- **THEN** the server responds with `200 OK` and an empty body, and the Pixoo device receives a single-frame animation with the image resized to 64×64 pixels.

#### Scenario: Static PNG upload displays on Pixoo
- **WHEN** a client posts a valid PNG file to `/draw/upload`
- **THEN** the server responds with `200 OK` and the Pixoo device receives a single-frame animation with the image resized to 64×64 pixels.

#### Scenario: Static WebP upload displays on Pixoo
- **WHEN** a client posts a valid static WebP file to `/draw/upload`
- **THEN** the server responds with `200 OK` and the Pixoo device receives a single-frame animation with the image resized to 64×64 pixels.

### Requirement: Draw upload supports animated GIF and animated WebP
The system SHALL detect animated GIF and animated WebP files, extract each frame, resize every frame to 64×64 pixels, read each frame's delay, multiply it by the configured `animation_speed_factor`, and send the frames as a multi-frame animation to the Pixoo device.

#### Scenario: Animated GIF with multiple frames
- **WHEN** a client uploads an animated GIF with 10 frames, each having a 100ms delay, and `animation_speed_factor` is `1.4`
- **THEN** the server sends 10 frames to the Pixoo with `PicNum=10`, `PicOffset` incrementing from 0 to 9, each frame resized to 64×64, and `PicSpeed` set to 140ms per frame
- **AND** the response is `200 OK` with an empty body.

#### Scenario: Animated WebP with multiple frames
- **WHEN** a client uploads an animated WebP with 5 frames
- **THEN** the server extracts all 5 frames, resizes each to 64×64, applies the animation speed factor to each frame's delay, and sends them as a multi-frame animation to the Pixoo.

### Requirement: Draw upload caps animations at 60 frames
The system SHALL accept at most 60 frames (offsets 0–59) from an animated file. When the source contains more than 60 frames, the system SHALL use only the first 60 and log a warning indicating the original frame count and the truncated count.

#### Scenario: GIF with exactly 60 frames is accepted in full
- **WHEN** a client uploads an animated GIF with exactly 60 frames
- **THEN** all 60 frames are sent to the Pixoo with `PicNum=60` and `PicOffset` from 0 to 59
- **AND** no truncation warning is logged.

#### Scenario: GIF exceeding 60 frames is truncated with warning
- **WHEN** a client uploads an animated GIF with 90 frames
- **THEN** only the first 60 frames are sent to the Pixoo with `PicNum=60`
- **AND** a warning-level log entry is emitted indicating 90 frames were truncated to 60.

### Requirement: Draw upload enforces maximum image size
The system SHALL check the uploaded file size against the configured `max_image_size` before any image decoding. When the file exceeds the limit, the handler SHALL return `413 Payload Too Large` with a JSON body indicating the configured limit and the actual file size.

#### Scenario: Upload within size limit is processed
- **WHEN** a client uploads a 500 KB JPEG and `max_image_size` is 5 MB
- **THEN** the upload is accepted and processed normally.

#### Scenario: Upload exceeding size limit is rejected before decoding
- **WHEN** a client uploads a 6 MB GIF and `max_image_size` is 5 MB
- **THEN** the server responds with `413 Payload Too Large` and a JSON body containing the limit and actual size
- **AND** no image decoding is performed.

### Requirement: Draw upload validates file format
The system SHALL determine the uploaded file's format from the multipart part's content type header, falling back to magic-byte detection when the content type is missing or `application/octet-stream`. The system SHALL reject unsupported formats with `400 Bad Request` and a JSON body containing `"unsupported image format"`.

#### Scenario: Unsupported file type is rejected
- **WHEN** a client uploads a BMP file to `/draw/upload`
- **THEN** the server responds with `400 Bad Request` and a JSON error body indicating the image format is unsupported
- **AND** no Pixoo commands are sent.

#### Scenario: Missing content type falls back to magic-byte detection
- **WHEN** a client uploads a PNG file without a content type header on the multipart part
- **THEN** the server detects the format from magic bytes, decodes and resizes the image, and responds with `200 OK`.

### Requirement: Draw upload rejects missing or empty file field
The system SHALL return `400 Bad Request` with a validation error body when the multipart request does not contain a field named `file` or when the field is empty.

#### Scenario: Missing file field
- **WHEN** a client sends a multipart request to `/draw/upload` without a `file` field
- **THEN** the server responds with `400 Bad Request` and a validation error body.

#### Scenario: Empty file field
- **WHEN** a client sends a multipart request with an empty `file` field (zero bytes)
- **THEN** the server responds with `400 Bad Request` and a validation error body.

### Requirement: Draw remote endpoint downloads external images
The system SHALL provide a `POST /draw/remote` endpoint that accepts a JSON body containing a `link` field with an absolute HTTP or HTTPS URL. The handler SHALL reject invalid URLs with `400 Bad Request` and SHALL use the dedicated `RemoteFetcher` component (configured with `PIXOO_BRIDGE_REMOTE_TIMEOUT_MS`) to download the bytes into a bounded buffer that enforces `PIXOO_BRIDGE_MAX_IMAGE_SIZE`. Once downloaded, the handler SHALL guess the content type (using headers and magic bytes) and pass the bytes to the existing `decode_upload` + Pixoo command helper flow so that the downloaded animation is resized, composited, and framed exactly the same as `/draw/upload` before issuing `Draw/GetHttpGifId` and `Draw/SendHttpGif` commands.

#### Scenario: Remote PNG renders like upload
- **WHEN** a client posts `{"link":"https://images.example.com/logo.png"}` and the fetcher returns a valid PNG within size limits
- **THEN** the handler reuses `decode_upload` and the shared Pixoo command helpers, returns `200 OK`, and the Pixoo device receives the animation described by the downloaded frames.

#### Scenario: Remote host exceeds size limit
- **WHEN** the fetcher encounters a response whose `Content-Length` header or accumulated bytes exceed `PIXOO_BRIDGE_MAX_IMAGE_SIZE`
- **THEN** the handler aborts the download, returns `413 Payload Too Large` with the configured limit and actual byte count, and no Pixoo commands are sent.

#### Scenario: Remote fetch times out or fails TLS
- **WHEN** the fetcher cannot complete the download because of a timeout, TLS failure, or other network error
- **THEN** the handler returns `503 Service Unavailable` (with a body describing the underlying failure), logs the download error, and leaves the Pixoo client untouched (no retries).

#### Scenario: Invalid or unsupported link is rejected
- **WHEN** a client submits a `link` that is missing, not absolute, or uses a non-HTTP(S) scheme
- **THEN** the handler responds with `400 Bad Request` describing the validation error and DOES NOT attempt to download or call Pixoo commands.

### Requirement: Draw upload composites alpha against black background
The system SHALL composite any alpha channel against a black background before extracting RGB pixel data, so that transparent pixels render as black on the Pixoo display.

#### Scenario: PNG with transparency
- **WHEN** a client uploads a PNG with semi-transparent pixels
- **THEN** the alpha channel is composited against black before the RGB buffer is sent to the Pixoo.

### Requirement: Draw upload image processing module
The system SHALL provide an image processing module that accepts raw file bytes and a content type hint and returns a list of decoded frames, each containing a 64×64×3 RGB buffer and a delay in milliseconds. This module SHALL be independent of the HTTP layer and reusable by future endpoints.

#### Scenario: Module returns single frame for static image
- **WHEN** the module receives a JPEG file
- **THEN** it returns a single frame with a 64×64×3 byte RGB buffer.

#### Scenario: Module returns multiple frames for animated GIF
- **WHEN** the module receives an animated GIF with 5 frames
- **THEN** it returns 5 frames, each with a 64×64×3 byte RGB buffer and the frame's delay in milliseconds.

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

### Requirement: Draw text endpoint sends Pixoo text command
The system SHALL provide a `POST /draw/text` API that accepts a JSON body with `id`, `position`, `scrollDirection`, `font`, `textWidth`, `text`, `scrollSpeed`, `color`, and `textAlignment`, and SHALL send a `Draw/SendHttpText` command with the validated payload fields.

#### Scenario: Valid text request renders on Pixoo
- **WHEN** a client posts a valid text payload to `/draw/text`
- **THEN** the server responds with `200 OK` and sends `Draw/SendHttpText` with the same `LcdId`, `TextId`, position, font, width, scroll direction, speed, color, and alignment fields.

### Requirement: Draw text payload validation
The system SHALL validate text draw payloads with the following constraints before sending Pixoo commands:
- `id` (TextId) MUST be in the range 0-20 inclusive.
- `position.x` and `position.y` MUST be provided and be non-negative integers.
- `font` MUST be in the range 0-7 inclusive.
- `textWidth` MUST be in the range 16-64 inclusive.
- `text` MUST be a UTF-8 string with length no greater than 512.
- `scrollSpeed` MUST be in the range 0-100 inclusive.
- `scrollDirection` MUST be either `LEFT` or `RIGHT`.
- `textAlignment` MUST be `LEFT`, `MIDDLE`, or `RIGHT`.
- `color.red`, `color.green`, and `color.blue` MUST be in the range 0-255 inclusive.

#### Scenario: Invalid text payload is rejected
- **WHEN** a client posts a text payload with an out-of-range `id` or `text_width` to `/draw/text`
- **THEN** the server responds with `400 Bad Request` and a validation error body
- **AND** no Pixoo command is sent.

### Requirement: Draw clear text endpoint clears Pixoo text
The system SHALL provide a `POST /draw/text/clear` API that sends `Draw/ClearHttpText` to clear Pixoo text entries.

#### Scenario: Clear text request resets Pixoo text
- **WHEN** a client posts to `/draw/text/clear`
- **THEN** the server responds with `200 OK` and sends `Draw/ClearHttpText` to the Pixoo device.

### Requirement: Pixoo command failures are surfaced for text
The system SHALL map Pixoo command errors from `Draw/SendHttpText` and `Draw/ClearHttpText` to the same error responses used by other draw endpoints.

#### Scenario: Pixoo returns an error for text command
- **WHEN** the Pixoo device responds with a non-zero error code for `Draw/SendHttpText`
- **THEN** the server responds with the corresponding error status and body used for draw endpoints.
