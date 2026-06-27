# pixoo-bridge.rs

[![Conventional Commits](https://img.shields.io/badge/Conventional%20Commits-1.0.0-yellow.svg)](https://conventionalcommits.org)
![GitHub License](https://img.shields.io/github/license/simplyRoba/pixoo-bridge.rs?link=https%3A%2F%2Fgithub.com%2FsimplyRoba%2Fpixoo-bridge.rs%2Fblob%2Fmain%2FLICENSE)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/simplyRoba/pixoo-bridge.rs/ci.yml?link=https%3A%2F%2Fgithub.com%2FsimplyRoba%2Fpixoo-bridge.rs%2Factions%2Fworkflows%2Fci.yml%3Fquery%3Dbranch%253Amain)
[![GitHub release](https://img.shields.io/github/v/release/simplyRoba/pixoo-bridge.rs?link=https%3A%2F%2Fgithub.com%2FsimplyRoba%2Fpixoo-bridge.rs%2Freleases)](https://github.com/simplyRoba/pixoo-bridge.rs/releases)
[![GitHub issues](https://img.shields.io/github/issues/simplyRoba/pixoo-bridge.rs?link=https%3A%2F%2Fgithub.com%2FsimplyRoba%2Fpixoo-bridge.rs%2Fissues)](https://github.com/simplyRoba/pixoo-bridge.rs/issues)
![GitHub Repo stars](https://img.shields.io/github/stars/simplyRoba/pixoo-bridge.rs)

pixoo-bridge.rs consumes the Pixoo LED matrix's proprietary protocol and re-exposes its own HTTP API so orchestration systems (Home Assistant, automation platforms, etc.) can control the matrix easily without touching the vendor's ugly API.

## Quick Start

Replace the IP with your Pixoo's address and run:

```shell
docker run -p 4000:4000 -e "PIXOO_BASE_URL=http://xxx.xxx.xxx.xxx" ghcr.io/simplyroba/pixoo-bridge.rs:latest
```

or use the [docker-compose.yaml](/docker-compose.yaml):

```shell
docker compose up -d
```

The bridge exits on startup if `PIXOO_BASE_URL` is missing or invalid so misconfigurations fail fast.

On startup the container logs the resolved configuration (health forwarding flag, Pixoo base URL, and listener address). The bridge binds to port `4000` by default and honors `PIXOO_BRIDGE_PORT` when provided; make sure your container/service maps that port as needed.

## Configuration

| Variable | Required | Default | Description |
| --- | --- | --- | --- |
| `PIXOO_BASE_URL` | yes | - | Base URL for the Pixoo device, for example `http://<ip>`. |
| `PIXOO_ANIMATION_SPEED_FACTOR` | no | `1.4` | Multiplier applied to animation frame delays read from GIF/WebP files. Values > 1 slow down, < 1 speed up. |
| `PIXOO_BRIDGE_HEALTH_FORWARD` | no | `true` | `true`/`false` to control whether `/health` cascades to the device. |
| `PIXOO_BRIDGE_LOG_LEVEL` | no | `INFO` | Controls logging verbosity (`DEBUG`, `INFO`, `WARN`, `ERROR`). |
| `PIXOO_BRIDGE_REMOTE_TIMEOUT_MS` | no | `10000` | Request timeout (milliseconds) for all remote calls. |
| `PIXOO_BRIDGE_MAX_IMAGE_SIZE` | no | `5MB` | Maximum accepted image upload size. Accepts human-readable values like `5MB`, `128KB`. |
| `PIXOO_BRIDGE_PORT` | no | `4000` | HTTP listener port override that keeps container/network mappings aligned with runtime behavior. |

Unexpected Pixoo errors are logged with context; set `PIXOO_BRIDGE_LOG_LEVEL=DEBUG` to also see notable successes like health checks or retries that eventually succeed.

## API

Interactive API documentation (Swagger UI) is served at `/docs`; the raw OpenAPI specification is available at `/api-docs/openapi.json`. Opening the bridge root (`/`) redirects to `/docs`.

| Method | Endpoint | Description | Success | Client Errors |
| --- | --- | --- | --- | --- |
| `GET` | `/health` | Bridge health probe (cascades to device if enabled). | `200` | — |
| `POST` | `/reboot` | Request a Pixoo reboot. | `200` | — |
| `POST` | `/tools/timer/start` | Start timer. Body: `{ "minute": 0-59, "second": 0-59 }` | `200` | `400` invalid payload |
| `POST` | `/tools/timer/stop` | Stop the timer. | `200` | — |
| `POST` | `/tools/stopwatch/{action}` | Control stopwatch. Action: `start`, `stop`, `reset` | `200` | `400` invalid action |
| `POST` | `/tools/scoreboard` | Set scores. Body: `{ "blue_score": 0-999, "red_score": 0-999 }` | `200` | `400` out-of-range |
| `POST` | `/tools/soundmeter/{action}` | Control soundmeter. Action: `start`, `stop` | `200` | `400` invalid action |
| `POST` | `/draw/fill` | Fill the display with a single RGB color. Body: `{ "red": 0-255, "green": 0-255, "blue": 0-255 }` | `200` | `400` invalid payload |
| `POST` | `/draw/upload` | Upload an image (JPEG, PNG, WebP, GIF) to display. Multipart form with `file` field. Animated GIF/WebP supported (max 60 frames). | `200` | `400` invalid format/missing file, `413` file too large |
| `POST` | `/draw/remote` | Download an image from a URL to display. Body: `{ "link": "http(s)://..." }`. | `200` | `400` invalid URL, `413` payload too large, `503` download failed |
| `POST` | `/draw/text` | Draw text. Body: `{ "id": 0-20, "position": { "x": >=0, "y": >=0 }, "scrollDirection": "LEFT"|"RIGHT", "font": 0-7, "textWidth": 16-64, "scrollSpeed": 0-100, "text": "...", "color": { "red": 0-255, "green": 0-255, "blue": 0-255 }, "textAlignment": "LEFT"|"MIDDLE"|"RIGHT" }` | `200` | `400` invalid payload |
| `POST` | `/draw/text/clear` | Clear the Pixoo text layer. | `200` | — |
| `GET` | `/manage/settings` | Display settings (visibility, brightness, rotation, mirror, temp unit, clock ID). | `200` | — |
| `GET` | `/manage/time` | Device time as ISO-8601 UTC/local timestamps. | `200` | — |
| `GET` | `/manage/weather` | Weather data (temps, pressure, humidity, wind). | `200` | — |
| `POST` | `/manage/weather/location` | Set the device's longitude/latitude so weather calculations stay accurate; body `{ "longitude": -180.0-180.0, "latitude": -90.0-90.0 }`. | `200` | `400` invalid coordinates |
| `POST` | `/manage/time` | Update the device's UTC clock with the bridge's current time (no body). | `200` | `500` system clock unavailable |
| `POST` | `/manage/time/offset/{offset}` | Apply a timezone offset (GMT±N, `offset` between `-12` and `14`) via Pixoo's `Sys/TimeZone` command. | `200` | `400` invalid offset |
| `POST` | `/manage/time/mode/{mode}` | Set time display mode (`12h` or `24h`). | `200` | `400` invalid mode |
| `POST` | `/manage/weather/temperature-unit/{unit}` | Set temperature unit (`celsius` or `fahrenheit`). | `200` | `400` invalid unit |
| `POST` | `/manage/display/{action}` | Toggle the display power; action must be `on` or `off`. | `200` | `400` invalid action |
| `POST` | `/manage/display/brightness/{value}` | Set brightness from 0–100. | `200` | `400` invalid value |
| `POST` | `/manage/display/rotation/{angle}` | Rotate the screen; `angle` must be `0`, `90`, `180`, or `270`. | `200` | `400` invalid angle |
| `POST` | `/manage/display/mirror/{action}` | Enable/disable mirror mode via `on`/`off`. | `200` | `400` invalid action |
| `POST` | `/manage/display/brightness/overclock/{action}` | Enable or disable overclock mode (`on`/`off`). | `200` | `400` invalid action |
| `POST` | `/manage/display/white-balance` | Adjust RGB white balance; body `{ "red": 0-100, "green": 0-100, "blue": 0-100 }`. | `200` | `400` invalid payload |

### Error responses

Every error response (`4xx` and `5xx`) shares one canonical envelope. The root object always has exactly these three fields:

- `error_status` (int) — the HTTP status, mirrored into the body
- `error_kind` (string) — discriminator: one of `validation`, `not-found`, `payload-too-large`, `unreachable`, `timeout`, `device-error`, `remote-fetch`, `internal`
- `message` (string) — human-readable description

All case-specific data lives in a single optional `details` object, which is **omitted entirely when empty**:

- validation (`400`): `details` holds the per-field/per-action errors, e.g. `{ "red": ["range"] }`
- payload-too-large (`413`): `details` is `{ "limit": <int>, "actual": <int> }`
- device error (`503`): `details` is `{ "error_code": <int> }` when the device provided one
- not-found (`404`), timeouts, and unreachable: no `details` key

Example (device error):

```json
{
  "error_status": 503,
  "error_kind": "device-error",
  "message": "Pixoo Channel/SetBrightness command: device returned error_code 1",
  "details": { "error_code": 1 }
}
```

## Observability

Every HTTP response includes an `X-Request-Id` header. The bridge generates or forwards that identifier in middleware, carries it through tracing spans and Pixoo command logs, and echoes it in error responses so you can trace a single request from the client through the Pixoo device.

## Migration

### From pixoo-bridge (Kotlin)

This project is a drop-in replacement for the Kotlin-based [pixoo-bridge](https://github.com/simplyRoba/pixoo-bridge). The HTTP API is compatible so existing automation clients can switch by pointing at the new container.

**Configuration changes:**

| Kotlin (old) | Rust (new) | Notes |
| --- | --- | --- |
| `PIXOO_SIZE` | removed | No longer needed. |
| `PIXOO_BRIDGE_DOCS_ENABLED` | removed | Swagger UI is always served at `/docs`. |
| — | `PIXOO_BRIDGE_REMOTE_TIMEOUT_MS` | New. Controls remote call timeout. |

**Endpoint changes:**

| Kotlin (old) | Rust (new) |
| --- | --- |
| `/tool/...` | `/tools/...` |

**Breaking change — unified error envelope:** all error responses now use the single envelope described under [Error responses](#error-responses). The legacy root `error` string field has been **removed** (its text now lives in `message`, and `error_kind` is the discriminator), and the previously root-level `limit`, `actual`, and `error_code` fields are now nested under `details`. Clients that parsed the old `{ "error": ... }` / root `limit`/`actual`/`error_code` shapes must switch to `message`/`error_kind` and read extras from `details`.

All other endpoints and request/response shapes remain the same.

## Limitations

The Pixoo Channel control API will not be implemented. Use the Divoom app for that functionality.

## Further Resources

- [Pixoo-64 product page](https://divoom.com/products/pixoo-64)
- [Official Divoom API documentation](http://doc.divoom-gz.com/web/#/12?page_id=191)

---

**This project is developed spec-driven with AI assistance, reviewed by a critical human.**
