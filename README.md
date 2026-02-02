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
| `PIXOO_BRIDGE_HEALTH_FORWARD` | no | `true` | `true`/`false` to control whether `/health` cascades to the device. |
| `PIXOO_BRIDGE_LOG_LEVEL` | no | `info` | Controls logging verbosity (`debug`, `info`, `warn`, `error`). |
| `PIXOO_BRIDGE_PORT` | no | `4000` | HTTP listener port override that keeps container/network mappings aligned with runtime behavior. |

On startup the container logs the resolved configuration (health forwarding flag, sanitized Pixoo base URL, and listener address). The bridge binds to port `4000` by default and honors `PIXOO_BRIDGE_PORT` when provided; make sure your container/service maps that port as needed. Unexpected Pixoo errors are logged with context; set `PIXOO_BRIDGE_LOG_LEVEL=debug` to also see notable successes like health checks or retries that eventually succeed.

Request logging now runs across the entire router so every HTTP call emits its method, path, status, and duration to the logs at `DEBUG` level. Keep the default `info` level for normal operation, and flip `PIXOO_BRIDGE_LOG_LEVEL=debug` when you need the access log entries.

## API

| Method | Endpoint | Description | Responses |
| --- | --- | --- | --- |
| `GET` | `/health` | Return the bridge health probe result. | `200 { "status": "ok" }` when healthy, `503` when the probe fails |
| `POST` | `/reboot` | Request a Pixoo reboot. | `204 No Content` on success, `503` when Pixoo rejects or is unreachable |
| `POST` | `/tools/timer/start` | Start the timer by supplying `{ "minute": <number>, "second": <number> }`. | `200` on success, `400` for invalid payloads, `503` when Pixoo rejects the command |
| `POST` | `/tools/timer/stop` | Stop the timer. | `200` on success, `503` when Pixoo rejects or is unavailable |
| `POST` | `/tools/stopwatch/{action}` | Control the stopwatch by sending `action`=`start`, `stop`, or `reset`. | `200` on success, `400` for invalid verbs, `503` when Pixoo rejects the request |
| `POST` | `/tools/scoreboard` | Set scores by supplying `{ "blue_score": 0..999, "red_score": 0..999 }`. | `200` on success, `400` for out-of-range scores, `503` when Pixoo rejects the update |
| `POST` | `/tools/soundmeter/{action}` | Control the soundmeter with `action`=`start` or `stop`. | `200` on success, `400` for invalid verbs, `503` when Pixoo rejects the request |
| `GET` | `/manage/settings` | Return derived display settings (visibility, brightness, rotation, mirror, temperature unit, and clock ID) from `Channel/GetAllConf`. | `200` with typed JSON, `503` when Pixoo is unreachable |
| `GET` | `/manage/time` | Provide normalized UTC/local timestamps derived from `Device/GetDeviceTime`. | `200` with ISO-8601 strings, `503` on failure |
| `GET` | `/manage/weather` | Report normalized weather data (current/min/max temperatures, pressure, humidity, wind speed, and tracked weather string) without mutating Pixoo's original values. | `200` with typed JSON, `503` when Pixoo rejects the command |

The HTTP handlers for system maintenance now live in a dedicated `routes/system` module so `/health` and `/reboot` share the same middleware and routing surface while keeping `main.rs` lean.

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
