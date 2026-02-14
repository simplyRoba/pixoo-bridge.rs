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

All endpoints may return `502` (unreachable), `503` (device error), or `504` (timeout) with a JSON body: `{ "error_status", "message", "error_kind", "error_code?" }`.

## Observability

Every HTTP response includes an `X-Request-Id` header. The bridge generates or forwards that identifier in middleware, carries it through tracing spans and Pixoo command logs, and echoes it in error responses so you can trace a single request from the client through the Pixoo device.

## Migration

### From pixoo-bridge (Kotlin)

This project is a drop-in replacement for the Kotlin-based [pixoo-bridge](https://github.com/simplyRoba/pixoo-bridge). The HTTP API is compatible so existing automation clients can switch by pointing at the new container.

**Configuration changes:**

| Kotlin (old) | Rust (new) | Notes |
| --- | --- | --- |
| `PIXOO_SIZE` | removed | No longer needed. |
| `PIXOO_BRIDGE_DOCS_ENABLED` | removed | No built-in API docs UI. |
| — | `PIXOO_BRIDGE_REMOTE_TIMEOUT_MS` | New. Controls remote call timeout. |

**Endpoint changes:**

| Kotlin (old) | Rust (new) |
| --- | --- |
| `/tool/...` | `/tools/...` |

All other endpoints and request/response shapes remain the same.

## Limitations

The Pixoo Channel control API will not be implemented. Use the Divoom app for that functionality.

## Further Resources

- [Pixoo-64 product page](https://divoom.com/products/pixoo-64)
- [Official Divoom API documentation](http://doc.divoom-gz.com/web/#/12?page_id=191)

---

# Contributing

If you want to build or contribute, this project targets a minimal Rust service that bridges Pixoo device protocols to a more usable HTTP interface.

## Tech Stack

- Rust (stable toolchain)
- [axum](https://github.com/tokio-rs/axum) HTTP framework on [tokio](https://tokio.rs/)
- Multi-arch Docker images (`linux/amd64`, `linux/arm64`)

## Development

Run `cargo fmt && cargo clippy && cargo test` before committing. Follow conventional commit format.

## Releases

Releases are managed by [release-please](https://github.com/googleapis/release-please). Merging to `main` automatically creates or updates a release PR. Once that PR is merged, the pipeline compiles binaries for `linux/amd64` and `linux/arm64`, uploads them as GitHub release assets, and publishes a multi-arch Docker image to `ghcr.io`.

---

**This project is developed spec-driven with AI assistance, reviewed by a critical human.**
