# pixoo-bridge.rs

![Under Heavy Development](https://img.shields.io/badge/status-under%20heavy%20development-red)
![AI Assisted](https://img.shields.io/badge/development-AI%20assisted-blue)

pixoo-bridge.rs consumes the Pixoo LED matrix's proprietary protocol and re‑exposes its own HTTP API so orchestration systems (Home Assistant, automation platforms, etc.) can control the matrix easily without touching the vendor's ugly API.

## Tech Stack

- Rust (stable toolchain via `cargo`)
- Native networking (HTTP/UDP) 
- Docker image for deployment
- Minimal runtime footprint

## Usage

A standalone Rust bridge service that translates simple HTTP commands into Pixoo device protocol. Perfect for automation systems like Home Assistant.

## Development

Run `cargo fmt && cargo test` before committing. Follow conventional commit format.

## Deployment

Docker image published to GitHub Container Registry (GHCR).

---

**This project is developed with AI assistance.**