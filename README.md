# pixoo-bridge.rs

![Under Heavy Development](https://img.shields.io/badge/status-under%20heavy%20development-red)
![AI Assisted](https://img.shields.io/badge/development-AI%20assisted-blue)

pixoo-bridge.rs consumes the Pixoo LED matrix's proprietary protocol and re‑exposes its own HTTP API so orchestration systems (Home Assistant, automation platforms, etc.) can control the matrix easily without touching the vendor's ugly API.

## Usage

This project is under heavy development and does not provide user-facing functionality yet. It aims to become a simple bridge for controlling a Pixoo matrix without vendor tooling.

### Healthcheck

The bridge exposes `GET /health` for container probes. On success it returns HTTP 200 with `{"status":"ok"}`. When `PIXOO_BRIDGE_HEALTH_FORWARD` is enabled (default), the bridge will call the Pixoo device `/get` endpoint and return HTTP 503 if the device is unhealthy.

Environment variables:
- `PIXOO_BRIDGE_HEALTH_FORWARD`: `true`/`false`, defaults to `true`.
- `PIXOO_DEVICE_IP`: IP address of the Pixoo device used for health forwarding.

## Contributing

If you want to build or contribute, this project targets a minimal Rust service that bridges Pixoo device protocols to a more usable HTTP interface.

### Tech Stack

- Rust (stable toolchain via `cargo`)
- Native networking (HTTP/UDP)
- Docker image for deployment
- Minimal runtime footprint

### Development

Run `cargo fmt && cargo clippy && cargo test` before committing. Follow conventional commit format.

---

**This project is developed spec driven with AI assistance, reviewed by a critical human.**
