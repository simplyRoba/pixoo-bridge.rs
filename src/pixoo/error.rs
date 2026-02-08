use crate::request_id::RequestId;
use axum::{http::StatusCode, Json};
use serde::Serialize;
use serde_json::Value;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum PixooError {
    Http(reqwest::Error),
    HttpStatus(u16),
    InvalidBaseUrl(String),
    InvalidResponse(String),
    MissingErrorCode,
    InvalidErrorCode(Value),
    DeviceError { code: i64, payload: Value },
}

impl fmt::Display for PixooError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PixooError::Http(err) => write!(formatter, "http error: {err}"),
            PixooError::HttpStatus(status) => write!(formatter, "unexpected HTTP status: {status}"),
            PixooError::InvalidBaseUrl(message) => {
                write!(formatter, "invalid base url: {message}")
            }
            PixooError::InvalidResponse(message) => {
                write!(formatter, "invalid response: {message}")
            }
            PixooError::MissingErrorCode => formatter.write_str("missing error_code in response"),
            PixooError::InvalidErrorCode(value) => {
                write!(formatter, "invalid error_code value: {value}")
            }
            PixooError::DeviceError { code, .. } => {
                write!(formatter, "device returned error_code {code}")
            }
        }
    }
}

impl Error for PixooError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PixooError::Http(err) => Some(err),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for PixooError {
    fn from(err: reqwest::Error) -> Self {
        PixooError::Http(err)
    }
}

impl PixooError {
    pub fn http_status(&self) -> Option<u16> {
        match self {
            PixooError::HttpStatus(status) => Some(*status),
            _ => None,
        }
    }

    pub fn error_code(&self) -> Option<i64> {
        match self {
            PixooError::DeviceError { code, .. } => Some(*code),
            _ => None,
        }
    }

    pub fn payload(&self) -> Option<&Value> {
        match self {
            PixooError::DeviceError { payload, .. } => Some(payload),
            _ => None,
        }
    }

    pub fn category(&self) -> PixooErrorCategory {
        match self {
            PixooError::Http(err) => {
                if err.is_timeout() {
                    PixooErrorCategory::Timeout
                } else if err.is_connect() {
                    PixooErrorCategory::Unreachable
                } else {
                    PixooErrorCategory::DeviceError
                }
            }
            PixooError::HttpStatus(_)
            | PixooError::DeviceError { .. }
            | PixooError::InvalidResponse(_)
            | PixooError::MissingErrorCode
            | PixooError::InvalidErrorCode(_) => PixooErrorCategory::DeviceError,
            PixooError::InvalidBaseUrl(_) => PixooErrorCategory::Unknown,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixooErrorCategory {
    Unreachable,
    Timeout,
    DeviceError,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PixooHttpErrorKind {
    Unreachable,
    Timeout,
    DeviceError,
}

#[derive(Debug, Serialize)]
pub struct PixooHttpErrorResponse {
    pub error_status: u16,
    pub message: String,
    pub error_kind: PixooHttpErrorKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<i64>,
}

pub fn map_pixoo_error(
    error: &PixooError,
    context: &str,
    request_id: Option<&RequestId>,
) -> (StatusCode, Json<PixooHttpErrorResponse>) {
    let kind = match error.category() {
        PixooErrorCategory::Unreachable => PixooHttpErrorKind::Unreachable,
        PixooErrorCategory::Timeout => PixooHttpErrorKind::Timeout,
        _ => PixooHttpErrorKind::DeviceError,
    };

    let status = match kind {
        PixooHttpErrorKind::Unreachable => StatusCode::BAD_GATEWAY,
        PixooHttpErrorKind::Timeout => StatusCode::GATEWAY_TIMEOUT,
        PixooHttpErrorKind::DeviceError => StatusCode::SERVICE_UNAVAILABLE,
    };

    let message = match request_id {
        Some(id) => format!("{context}: {error} (request_id={id})"),
        None => format!("{context}: {error}"),
    };

    let payload = PixooHttpErrorResponse {
        error_status: status.as_u16(),
        message,
        error_kind: kind,
        error_code: error.error_code(),
    };

    (status, Json(payload))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pixoo::{PixooClient, PixooClientConfig, PixooCommand};
    use httpmock::Method::POST;
    use httpmock::MockServer;
    use serde_json::{Map, Value};
    use std::time::Duration;
    use tokio::net::TcpListener;

    async fn free_address() -> String {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);
        format!("http://{addr}")
    }

    #[tokio::test]
    async fn unreachable_error_maps_to_bad_gateway() {
        let base_url = free_address().await;
        let client = PixooClient::new(base_url, PixooClientConfig::default()).expect("client");
        let err = client
            .send_command(PixooCommand::ToolsTimer, Map::<String, Value>::new())
            .await
            .expect_err("expected unreachable");

        let (status, body) = map_pixoo_error(&err, "test unreachable", None);
        let response = body.0;

        assert_eq!(status, StatusCode::BAD_GATEWAY);
        assert_eq!(response.error_kind, PixooHttpErrorKind::Unreachable);
        assert!(response.error_code.is_none());
    }

    #[tokio::test]
    async fn device_error_maps_to_service_unavailable() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(POST).path("/post");
            then.status(200).body(r#"{"error_code":1}"#);
        });

        let base_url = server.base_url();
        let client = PixooClient::new(base_url, PixooClientConfig::default()).expect("client");
        let err = client
            .send_command(PixooCommand::ToolsTimer, Map::<String, Value>::new())
            .await
            .expect_err("expected device error");

        let (status, body) = map_pixoo_error(&err, "test device", None);
        let response = body.0;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(response.error_kind, PixooHttpErrorKind::DeviceError);
        assert_eq!(response.error_code, Some(1));
    }

    #[tokio::test]
    async fn timeout_error_maps_to_gateway_timeout() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let addr = listener.local_addr().unwrap();
        let _handle = tokio::spawn(async move {
            if let Ok((socket, _)) = listener.accept().await {
                tokio::time::sleep(Duration::from_secs(1)).await;
                drop(socket);
            }
        });

        let config =
            PixooClientConfig::new(Duration::from_millis(50), 2, Duration::from_millis(200));
        let client = PixooClient::new(format!("http://{addr}"), config).expect("client");
        let err = client
            .send_command(PixooCommand::ToolsTimer, Map::<String, Value>::new())
            .await
            .expect_err("expected timeout");

        let (status, body) = map_pixoo_error(&err, "test timeout", None);
        let response = body.0;

        assert_eq!(status, StatusCode::GATEWAY_TIMEOUT);
        assert_eq!(response.error_kind, PixooHttpErrorKind::Timeout);
    }
}
