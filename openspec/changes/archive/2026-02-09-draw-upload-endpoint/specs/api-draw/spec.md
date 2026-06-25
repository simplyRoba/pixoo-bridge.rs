## ADDED Requirements

### Requirement: Draw upload endpoint accepts image files via multipart upload
The system SHALL provide a `POST /draw/upload` API that accepts `multipart/form-data` with a field named `file` containing a JPEG, PNG, WebP, or GIF image. The handler SHALL decode the image, resize it to 64×64 pixels, and send it to the Pixoo device. On success the handler SHALL return `200 OK` with an empty body.

#### Scenario: Static JPEG upload displays on Pixoo
- **WHEN** a client posts a valid JPEG file to `/draw/upload` as multipart form data with field name `file`
- **THEN** the server responds with `200 OK` and an empty body, and the Pixoo device receives a single-frame animation with the image resized to 64×64 pixels

#### Scenario: Static PNG upload displays on Pixoo
- **WHEN** a client posts a valid PNG file to `/draw/upload`
- **THEN** the server responds with `200 OK` and the Pixoo device receives a single-frame animation with the image resized to 64×64 pixels

#### Scenario: Static WebP upload displays on Pixoo
- **WHEN** a client posts a valid static WebP file to `/draw/upload`
- **THEN** the server responds with `200 OK` and the Pixoo device receives a single-frame animation with the image resized to 64×64 pixels

### Requirement: Draw upload supports animated GIF and animated WebP
The system SHALL detect animated GIF and animated WebP files, extract each frame, resize every frame to 64×64 pixels, read each frame's delay, multiply it by the configured `animation_speed_factor`, and send the frames as a multi-frame animation to the Pixoo device.

#### Scenario: Animated GIF with multiple frames
- **WHEN** a client uploads an animated GIF with 10 frames, each having a 100ms delay, and `animation_speed_factor` is `1.4`
- **THEN** the server sends 10 frames to the Pixoo with `PicNum=10`, `PicOffset` incrementing from 0 to 9, each frame resized to 64×64, and `PicSpeed` set to 140ms per frame
- **AND** the response is `200 OK` with an empty body

#### Scenario: Animated WebP with multiple frames
- **WHEN** a client uploads an animated WebP with 5 frames
- **THEN** the server extracts all 5 frames, resizes each to 64×64, applies the animation speed factor to each frame's delay, and sends them as a multi-frame animation to the Pixoo

### Requirement: Draw upload caps animations at 60 frames
The system SHALL accept at most 60 frames (offsets 0–59) from an animated file. When the source contains more than 60 frames, the system SHALL use only the first 60 and log a warning indicating the original frame count and the truncated count.

#### Scenario: GIF with exactly 60 frames is accepted in full
- **WHEN** a client uploads an animated GIF with exactly 60 frames
- **THEN** all 60 frames are sent to the Pixoo with `PicNum=60` and `PicOffset` from 0 to 59
- **AND** no truncation warning is logged

#### Scenario: GIF exceeding 60 frames is truncated with warning
- **WHEN** a client uploads an animated GIF with 90 frames
- **THEN** only the first 60 frames are sent to the Pixoo with `PicNum=60`
- **AND** a warning-level log entry is emitted indicating 90 frames were truncated to 60

### Requirement: Draw upload enforces maximum image size
The system SHALL check the uploaded file size against the configured `max_image_size` before any image decoding. When the file exceeds the limit, the handler SHALL return `413 Payload Too Large` with a JSON body indicating the configured limit and the actual file size.

#### Scenario: Upload within size limit is processed
- **WHEN** a client uploads a 500 KB JPEG and `max_image_size` is 5 MB
- **THEN** the upload is accepted and processed normally

#### Scenario: Upload exceeding size limit is rejected before decoding
- **WHEN** a client uploads a 6 MB GIF and `max_image_size` is 5 MB
- **THEN** the server responds with `413 Payload Too Large` and a JSON body containing the limit and actual size
- **AND** no image decoding is performed

### Requirement: Draw upload validates file format
The system SHALL determine the uploaded file's format from the multipart part's content type header, falling back to magic-byte detection when the content type is missing or `application/octet-stream`. The system SHALL reject unsupported formats with `400 Bad Request` and a JSON body containing `"unsupported image format"`.

#### Scenario: Unsupported file type is rejected
- **WHEN** a client uploads a BMP file to `/draw/upload`
- **THEN** the server responds with `400 Bad Request` and a JSON error body indicating the image format is unsupported
- **AND** no Pixoo commands are sent

#### Scenario: Missing content type falls back to magic-byte detection
- **WHEN** a client uploads a PNG file without a content type header on the multipart part
- **THEN** the server detects the format from magic bytes, decodes and resizes the image, and responds with `200 OK`

### Requirement: Draw upload rejects missing or empty file field
The system SHALL return `400 Bad Request` with a validation error body when the multipart request does not contain a field named `file` or when the field is empty.

#### Scenario: Missing file field
- **WHEN** a client sends a multipart request to `/draw/upload` without a `file` field
- **THEN** the server responds with `400 Bad Request` and a validation error body

#### Scenario: Empty file field
- **WHEN** a client sends a multipart request with an empty `file` field (zero bytes)
- **THEN** the server responds with `400 Bad Request` and a validation error body

### Requirement: Draw upload composites alpha against black background
The system SHALL composite any alpha channel against a black background before extracting RGB pixel data, so that transparent pixels render as black on the Pixoo display.

#### Scenario: PNG with transparency
- **WHEN** a client uploads a PNG with semi-transparent pixels
- **THEN** the alpha channel is composited against black before the RGB buffer is sent to the Pixoo

### Requirement: Draw upload image processing module
The system SHALL provide an image processing module that accepts raw file bytes and a content type hint and returns a list of decoded frames, each containing a 64×64×3 RGB buffer and a delay in milliseconds. This module SHALL be independent of the HTTP layer and reusable by future endpoints.

#### Scenario: Module returns single frame for static image
- **WHEN** the module receives a JPEG file
- **THEN** it returns a single frame with a 64×64×3 byte RGB buffer

#### Scenario: Module returns multiple frames for animated GIF
- **WHEN** the module receives an animated GIF with 5 frames
- **THEN** it returns 5 frames, each with a 64×64×3 byte RGB buffer and the frame's delay in milliseconds

## MODIFIED Requirements

### Requirement: Pixoo command flow for draw fill
The draw handler SHALL internally fetch a fresh animation ID via `Draw/GetHttpGifId` before issuing a single-frame automation with `Draw/SendHttpGif`. The automation arguments SHALL set `PicNum=1`, `PicOffset=0`, `PicWidth=64`, and include the previously fetched `PicId`. The GIF ID retrieval and frame-sending logic SHALL be implemented as shared helpers reusable by both `/draw/fill` and `/draw/upload`.

#### Scenario: Pixoo commands are sequenced correctly
- **WHEN** the handler receives a valid fill request
- **THEN** it first calls `Draw/GetHttpGifId`, then calls `Draw/SendHttpGif` with the metadata described above, and the response is `error_code=0` for both commands

#### Scenario: Upload endpoint reuses the same Pixoo command helpers
- **WHEN** the upload handler processes a valid image
- **THEN** it uses the same shared GIF ID retrieval and frame-sending helpers as `/draw/fill`, benefiting from the same error handling and response parsing
