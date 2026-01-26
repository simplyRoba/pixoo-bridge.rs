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

## Pixoo Client

The bridge now provides a Pixoo HTTP client that always POSTs JSON bodies and parses JSON responses even if the device responds with the wrong content type. Successful responses return the remaining fields after `error_code` validation.

```rust
use pixoo_bridge::pixoo::{PixooClient, PixooCommand};
use serde_json::{Map, Value};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = PixooClient::from_ip("192.168.1.50")?;

let mut args = Map::new();
args.insert("Index".to_string(), Value::from(2));

let response = client
    .send_command(PixooCommand::ChannelSetCloudIndex, args)
    .await?;
println!("response fields: {response:?}");
# Ok(())
# }
```

Notes:
- All requests are HTTP POST with `Content-Type: application/json`.
- Responses are parsed as JSON regardless of response headers.
- Non-zero `error_code` values are returned as errors.
- Pixoo devices commonly accept commands at `/post`, and `PixooClient::from_ip` uses that default path.

## Development

Run `cargo fmt && cargo test` before committing. Follow conventional commit format.

## Deployment

Docker image published to GitHub Container Registry (GHCR).

---

**This project is developed with AI assistance.**
