# pixoo-bridge.rs

![Under Heavy Development](https://img.shields.io/badge/status-under%20heavy%20development-red)
![AI Assisted](https://img.shields.io/badge/development-AI%20assisted-blue)

pixoo-bridge.rs consumes the Pixoo LED matrix's proprietary protocol and re‑exposes its own HTTP API so orchestration systems (Home Assistant, automation platforms, etc.) can control the matrix easily without touching the vendor's ugly API.

## Usage

This project is under heavy development and does not provide user-facing functionality yet. It aims to become a simple bridge for controlling a Pixoo matrix without vendor tooling.

## Configuration

| Variable | Required | Default | Description |
| --- | --- | --- | --- |
| `PIXOO_BASE_URL` | yes | - | Base URL for the Pixoo device, for example `http://<ip>`. |
| `PIXOO_TIMEOUT_MS` | no | `10000` | Request timeout (milliseconds) for upstream Pixoo calls; reducing this value forces quicker failures during automated checks. |
| `PIXOO_BRIDGE_HEALTH_FORWARD` | no | `true` | `true`/`false` to control whether `/health` cascades to the device. |
| `PIXOO_BRIDGE_LOG_LEVEL` | no | `info` | Controls logging verbosity (`debug`, `info`, `warn`, `error`). |
| `PIXOO_BRIDGE_PORT` | no | `4000` | HTTP listener port override that keeps container/network mappings aligned with runtime behavior. |

The bridge exits on startup if `PIXOO_BASE_URL` is missing or invalid so misconfigurations fail fast.

On startup the container logs the resolved configuration (health forwarding flag, Pixoo base URL, and listener address). The bridge binds to port `4000` by default and honors `PIXOO_BRIDGE_PORT` when provided; make sure your container/service maps that port as needed. Unexpected Pixoo errors are logged with context; set `PIXOO_BRIDGE_LOG_LEVEL=debug` to also see notable successes like health checks or retries that eventually succeed.

## API

| Method | Endpoint | Description | Success | Client Errors |
| --- | --- | --- | --- | --- |
| `GET` | `/health` | Bridge health probe (cascades to device if enabled). | `200` | — |
| `POST` | `/reboot` | Request a Pixoo reboot. | `204` | — |
| `POST` | `/tools/timer/start` | Start timer. Body: `{ "minute": 0-59, "second": 0-59 }` | `200` | `400` invalid payload |
| `POST` | `/tools/timer/stop` | Stop the timer. | `200` | — |
| `POST` | `/tools/stopwatch/{action}` | Control stopwatch. Action: `start`, `stop`, `reset` | `200` | `400` invalid action |
| `POST` | `/tools/scoreboard` | Set scores. Body: `{ "blue_score": 0-999, "red_score": 0-999 }` | `200` | `400` out-of-range |
| `POST` | `/tools/soundmeter/{action}` | Control soundmeter. Action: `start`, `stop` | `200` | `400` invalid action |
| `POST` | `/draw/fill` | Fill the display with a single RGB color. Body: `{ "red": 0-255, "green": 0-255, "blue": 0-255 }` | `200` | `400` invalid payload |
| `GET` | `/manage/settings` | Display settings (visibility, brightness, rotation, mirror, temp unit, clock ID). | `200` | — |
| `GET` | `/manage/time` | Device time as ISO-8601 UTC/local timestamps. | `200` | — |
| `GET` | `/manage/weather` | Weather data (temps, pressure, humidity, wind). | `200` | — |
| `POST` | `/manage/weather/location` | Set the device’s longitude/latitude so weather calculations stay accurate; body `{ "longitude": -180.0-180.0, "latitude": -90.0-90.0 }`. | `200` | `400` invalid coordinates |
| `POST` | `/manage/time` | Update the device’s UTC clock with the bridge’s current time (no body). | `200` | `500` system clock unavailable |
| `POST` | `/manage/time/offset/{offset}` | Apply a timezone offset (GMT±N, `offset` between `-12` and `14`) via Pixoo’s `Sys/TimeZone` command. | `200` | `400` invalid offset |
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

Every HTTP response now includes an `X-Request-Id` header. The bridge generates or forwards that identifier in middleware, carries it through tracing spans and Pixoo command logs, and echoes it in error responses so you can trace a single request from the client through the Pixoo device.

## Contributing

If you want to build or contribute, this project targets a minimal Rust service that bridges Pixoo device protocols to a more usable HTTP interface.

### Tech Stack

- Rust (stable toolchain via `cargo`)
- Native networking (HTTP/UDP)
- Docker image for deployment
- Minimal runtime footprint

### Development

Run `cargo fmt && cargo clippy && cargo test` before committing. Follow conventional commit format.

## Releases

Release binaries for `linux/amd64` and `linux/arm64` are now compiled in `publish-release.yml` using the same commands the Docker image expects. The workflow uploads those binaries as release assets, and the Dockerfile copies the matching prebuilt artifact for each `TARGETPLATFORM` so the container image no longer rebuilds the bridge.

## Migration

Projects upgrading from the legacy `pixoo-bridge` package should now use `pixoo.bridge.rs`. All existing automation clients are expected to replay their calls against the same routes, except for the tools namespace: `/tools/...` is the current endpoint surface and the singular `/tool/...` variants are no longer supported.

---

**This project is developed spec driven with AI assistance, reviewed by a critical human.**
