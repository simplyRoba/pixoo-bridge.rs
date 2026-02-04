use axum::{http::StatusCode, Json};
use serde::Serialize;

use crate::pixoo::{PixooError, PixooErrorCategory};

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

    let payload = PixooHttpErrorResponse {
        error_status: status.as_u16(),
        message: format!("{context}: {error}"),
        error_kind: kind,
        error_code: error.error_code(),
    };

    (status, Json(payload))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pixoo::{PixooClient, PixooCommand};
    use httpmock::Method::POST;
    use httpmock::MockServer;
    use serde_json::{Map, Value};
    use std::env;
    use std::time::Duration;
    use tokio::net::TcpListener;

    async fn free_address() -> String {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);
        format!("http://{addr}")
    }

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: Option<&str>) -> Self {
            let previous = env::var(key).ok();
            match value {
                Some(val) => env::set_var(key, val),
                None => env::remove_var(key),
            }
            EnvVarGuard { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(value) => env::set_var(self.key, value),
                None => env::remove_var(self.key),
            }
        }
    }

    #[tokio::test]
    async fn unreachable_error_maps_to_bad_gateway() {
        let base_url = free_address().await;
        let client = PixooClient::new(base_url).expect("client");
        let err = client
            .send_command(PixooCommand::ToolsTimer, Map::<String, Value>::new())
            .await
            .expect_err("expected unreachable");

        let (status, body) = map_pixoo_error(&err, "test unreachable");
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
        let client = PixooClient::new(base_url).expect("client");
        let err = client
            .send_command(PixooCommand::ToolsTimer, Map::<String, Value>::new())
            .await
            .expect_err("expected device error");

        let (status, body) = map_pixoo_error(&err, "test device");
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
                let _ = tokio::time::sleep(Duration::from_secs(1)).await;
                drop(socket);
            }
        });

        let _guard = EnvVarGuard::set("PIXOO_CLIENT_TIMEOUT_MS", Some("50"));
        let client = PixooClient::new(format!("http://{addr}")).expect("client");
        let err = client
            .send_command(PixooCommand::ToolsTimer, Map::<String, Value>::new())
            .await
            .expect_err("expected timeout");

        let (status, body) = map_pixoo_error(&err, "test timeout");
        let response = body.0;

        assert_eq!(status, StatusCode::GATEWAY_TIMEOUT);
        assert_eq!(response.error_kind, PixooHttpErrorKind::Timeout);
    }
}
