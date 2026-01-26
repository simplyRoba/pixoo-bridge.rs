# pixoo-bridge.rs

![Under Heavy Development](https://img.shields.io/badge/status-under%20heavy%20development-red)
![AI Assisted](https://img.shields.io/badge/development-AI%20assisted-blue)

pixoo-bridge.rs consumes the Pixoo LED matrix's proprietary protocol and re‑exposes its own HTTP API so orchestration systems (Home Assistant, automation platforms, etc.) can control the matrix easily without touching the vendor's ugly API.

## Usage

This project is under heavy development and does not provide user-facing functionality yet. It aims to become a simple bridge for controlling a Pixoo matrix without vendor tooling.

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

**This project is developed with AI assistance.**
