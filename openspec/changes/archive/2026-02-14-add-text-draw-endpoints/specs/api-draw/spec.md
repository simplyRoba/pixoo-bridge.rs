## ADDED Requirements

### Requirement: Draw text endpoint sends Pixoo text command
The system SHALL provide a `POST /draw/text` API that accepts a JSON body with `id`, `position`, `scrollDirection`, `font`, `textWidth`, `text`, `scrollSpeed`, `color`, and `textAlignment`, and SHALL send a `Draw/SendHttpText` command with the validated payload fields.

#### Scenario: Valid text request renders on Pixoo
- **WHEN** a client posts a valid text payload to `/draw/text`
- **THEN** the server responds with `200 OK` and sends `Draw/SendHttpText` with the same `LcdId`, `TextId`, position, font, width, scroll direction, speed, color, and alignment fields.

### Requirement: Draw text payload validation
The system SHALL validate text draw payloads with the following constraints before sending Pixoo commands:
- `id` (TextId) MUST be in the range 0–20 inclusive.
- `position.x` and `position.y` MUST be provided and be non-negative integers.
- `font` MUST be in the range 0–7 inclusive.
- `textWidth` MUST be in the range 16–64 inclusive.
- `text` MUST be a UTF-8 string with length no greater than 512.
- `scrollSpeed` MUST be in the range 0–100 inclusive.
- `scrollDirection` MUST be either `LEFT` or `RIGHT`.
- `textAlignment` MUST be `LEFT`, `MIDDLE`, or `RIGHT`.
- `color.red`, `color.green`, and `color.blue` MUST be in the range 0–255 inclusive.

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
