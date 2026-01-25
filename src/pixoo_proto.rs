use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PixooCommand {
    pub command: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PixooResponse {
    pub success: bool,
    pub data: serde_json::Value,
}

impl PixooCommand {
    pub fn new(command: &str, data: serde_json::Value) -> Self {
        Self {
            command: command.to_string(),
            data,
        }
    }

    pub fn to_bytes(&self) -> TransportResult<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| TransportError::Protocol(e.to_string()))
    }
}

use crate::transport::{TransportError, TransportResult};
