use axum::{http::StatusCode, Json};
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, Error)]
pub enum PixooError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("unexpected HTTP status: {0}")]
    HttpStatus(u16),

    #[error("invalid base url: {0}")]
    InvalidBaseUrl(String),

    #[error("invalid response: {0}")]
    InvalidResponse(String),

    #[error("missing error_code in response")]
    MissingErrorCode,

    #[error("invalid error_code value: {0}")]
    InvalidErrorCode(Value),

    #[error("device returned error_code {code}")]
    DeviceError { code: i64, payload: Value },
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

/// Discriminator for every error envelope.
///
/// `validation`, `not-found`, and `payload-too-large` describe request-side
/// failures; `unreachable`, `timeout`, and `device-error` originate from the
/// Pixoo device; `remote-fetch` covers failed remote image downloads; and
/// `internal` covers unexpected bridge-side failures (e.g. encoding or response
/// parsing).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum PixooHttpErrorKind {
    Validation,
    NotFound,
    PayloadTooLarge,
    Unreachable,
    Timeout,
    DeviceError,
    RemoteFetch,
    Internal,
}

/// Canonical error body for every error response (`4xx` and `5xx`).
///
/// All error responses share this shape so clients can rely on a single
/// envelope: the root is always `error_status`, `error_kind`, and `message`,
/// and any kind-specific data lives in the optional `details` object. `details`
/// is omitted entirely when there is no extra data; its per-kind contents are:
/// validation → a field/action error map; payload-too-large → `{ limit, actual }`;
/// device errors → `{ error_code }` when the device provided one.
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "error_status": 503,
    "error_kind": "device-error",
    "message": "Pixoo Channel/SetBrightness command: device returned error_code 1",
    "details": { "error_code": 1 }
}))]
pub struct PixooHttpErrorResponse {
    /// HTTP status mirrored into the body.
    pub error_status: u16,
    /// Failure category discriminator.
    pub error_kind: PixooHttpErrorKind,
    /// Human-readable failure description.
    pub message: String,
    /// Kind-specific data; omitted entirely when there is none.
    #[schema(value_type = Object)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl PixooHttpErrorResponse {
    /// Builds a canonical error response with the given status, kind, and
    /// message and no `details`.
    pub fn new(status: StatusCode, kind: PixooHttpErrorKind, message: impl Into<String>) -> Self {
        Self {
            error_status: status.as_u16(),
            error_kind: kind,
            message: message.into(),
            details: None,
        }
    }

    /// Builds a canonical error response carrying a kind-specific `details` object.
    pub fn with_details(
        status: StatusCode,
        kind: PixooHttpErrorKind,
        message: impl Into<String>,
        details: Value,
    ) -> Self {
        Self {
            error_status: status.as_u16(),
            error_kind: kind,
            message: message.into(),
            details: Some(details),
        }
    }

    /// Consumes the body into an axum response with the matching status code.
    pub fn into_response(self) -> axum::response::Response {
        use axum::response::IntoResponse;

        let status =
            StatusCode::from_u16(self.error_status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}

pub fn map_pixoo_error(
    error: &PixooError,
    context: &str,
) -> (StatusCode, Json<PixooHttpErrorResponse>) {
    let kind = match error.category() {
        PixooErrorCategory::Unreachable => PixooHttpErrorKind::Unreachable,
        PixooErrorCategory::Timeout => PixooHttpErrorKind::Timeout,
        _ => PixooHttpErrorKind::DeviceError,
    };

    let status = match kind {
        PixooHttpErrorKind::Unreachable => StatusCode::BAD_GATEWAY,
        PixooHttpErrorKind::Timeout => StatusCode::GATEWAY_TIMEOUT,
        // `kind` is always a device variant here; all remaining kinds map to 503.
        PixooHttpErrorKind::Validation
        | PixooHttpErrorKind::NotFound
        | PixooHttpErrorKind::PayloadTooLarge
        | PixooHttpErrorKind::DeviceError
        | PixooHttpErrorKind::RemoteFetch
        | PixooHttpErrorKind::Internal => StatusCode::SERVICE_UNAVAILABLE,
    };

    let message = format!("{context}: {error}");

    let details = error
        .error_code()
        .map(|code| serde_json::json!({ "error_code": code }));

    let payload = PixooHttpErrorResponse {
        error_status: status.as_u16(),
        error_kind: kind,
        message,
        details,
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
            .send_command(&PixooCommand::ToolsTimer, Map::<String, Value>::new())
            .await
            .expect_err("expected unreachable");

        let (status, body) = map_pixoo_error(&err, "test unreachable");
        let response = body.0;

        assert_eq!(status, StatusCode::BAD_GATEWAY);
        assert_eq!(response.error_kind, PixooHttpErrorKind::Unreachable);
        assert!(response.details.is_none());
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
            .send_command(&PixooCommand::ToolsTimer, Map::<String, Value>::new())
            .await
            .expect_err("expected device error");

        let (status, body) = map_pixoo_error(&err, "test device");
        let response = body.0;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(response.error_kind, PixooHttpErrorKind::DeviceError);
        assert_eq!(
            response.details,
            Some(serde_json::json!({ "error_code": 1 }))
        );
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
            .send_command(&PixooCommand::ToolsTimer, Map::<String, Value>::new())
            .await
            .expect_err("expected timeout");

        let (status, body) = map_pixoo_error(&err, "test timeout");
        let response = body.0;

        assert_eq!(status, StatusCode::GATEWAY_TIMEOUT);
        assert_eq!(response.error_kind, PixooHttpErrorKind::Timeout);
    }
}
