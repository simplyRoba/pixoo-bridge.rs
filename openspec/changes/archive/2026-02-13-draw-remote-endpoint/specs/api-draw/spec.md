## ADDED Requirements

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
