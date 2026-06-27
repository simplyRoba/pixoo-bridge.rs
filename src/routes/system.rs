use crate::pixoo::{map_pixoo_error, PixooCommand};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::{json, Map};
use std::sync::Arc;
use tracing::{debug, error};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::common::dispatch_pixoo_command;
use crate::pixoo::error::{DeviceErrorResponse, DeviceTimeoutResponse, DeviceUnreachableResponse};

use crate::state::AppState;

pub fn system_router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(health))
        .routes(routes!(reboot))
}

/// `200 OK` body for `/health`.
#[derive(Serialize, ToSchema)]
#[allow(dead_code)]
struct HealthStatus {
    /// Always `"ok"`.
    #[schema(example = "ok")]
    status: String,
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "system",
    responses(
        (status = 200, description = "Bridge is healthy (and Pixoo reachable when forwarding is enabled)", body = HealthStatus),
        (status = 502, response = DeviceUnreachableResponse),
        (status = 503, response = DeviceErrorResponse),
        (status = 504, response = DeviceTimeoutResponse)
    )
)]
#[tracing::instrument(skip(state))]
async fn health(State(state): State<Arc<AppState>>) -> Response {
    if !state.health_forward {
        return (StatusCode::OK, Json(json!({ "status": "ok" }))).into_response();
    }

    let client = &state.pixoo_client;
    match client.health_check().await {
        Ok(()) => {
            debug!("Forwarded health check to Pixoo succeeded");
            (StatusCode::OK, Json(json!({ "status": "ok" }))).into_response()
        }
        Err(err) => {
            let (status, body) = map_pixoo_error(&err, "Pixoo health check");
            error!(error = ?err, status = %status, "Pixoo health check failed");
            (status, body).into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/reboot",
    tag = "system",
    responses(
        (status = 200, description = "Reboot command accepted"),
        (status = 502, response = DeviceUnreachableResponse),
        (status = 503, response = DeviceErrorResponse),
        (status = 504, response = DeviceTimeoutResponse)
    )
)]
#[tracing::instrument(skip(state))]
async fn reboot(State(state): State<Arc<AppState>>) -> Response {
    dispatch_pixoo_command(&state, PixooCommand::SystemReboot, Map::new()).await
}

#[cfg(test)]
mod tests {
    use super::system_router;
    use crate::pixoo::{PixooClient, PixooClientConfig};
    use crate::state::AppState;
    use axum::body::{to_bytes, Body};
    use axum::http::{Method, Request, StatusCode};
    use axum::Router;
    use httpmock::{Method as MockMethod, MockServer};
    use std::sync::Arc;
    use tower::ServiceExt;

    fn build_system_app(state: Arc<AppState>) -> Router {
        let (router, _api) = system_router().with_state(state).split_for_parts();
        router
    }

    fn system_state(client: PixooClient, health_forward: bool) -> Arc<AppState> {
        let mut state = AppState::with_client(client);
        state.health_forward = health_forward;
        Arc::new(state)
    }

    async fn send_request(app: &Router, method: Method, uri: &str) -> (StatusCode, String) {
        let req = Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(req).await.unwrap();
        let status = response.status();
        let body_bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap_or_default();
        (status, String::from_utf8_lossy(&body_bytes).to_string())
    }

    #[tokio::test]
    async fn health_ok_when_forwarding_disabled() {
        let server = MockServer::start_async().await;
        let app = build_system_app(system_state(
            PixooClient::new(server.base_url(), PixooClientConfig::default()).expect("client"),
            false,
        ));

        let (status, body) = send_request(&app, Method::GET, "/health").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, r#"{"status":"ok"}"#);
    }

    #[tokio::test]
    async fn health_ok_when_pixoo_healthy() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(MockMethod::GET).path("/get");
            then.status(200);
        });

        let app = build_system_app(system_state(
            PixooClient::new(server.base_url(), PixooClientConfig::default()).expect("client"),
            true,
        ));

        let (status, body) = send_request(&app, Method::GET, "/health").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, r#"{"status":"ok"}"#);
    }

    #[tokio::test]
    async fn health_forwarding_contacts_pixoo_when_enabled() {
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(MockMethod::GET).path("/get");
            then.status(200);
        });

        let app = build_system_app(system_state(
            PixooClient::new(server.base_url(), PixooClientConfig::default()).expect("client"),
            true, // health_forward enabled
        ));

        let (status, _) = send_request(&app, Method::GET, "/health").await;

        assert_eq!(status, StatusCode::OK);
        mock.assert(); // Verify Pixoo was contacted
    }

    #[tokio::test]
    async fn health_reports_unhealthy_on_pixoo_failure() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(MockMethod::GET).path("/get");
            then.status(500);
        });

        let app = build_system_app(system_state(
            PixooClient::new(server.base_url(), PixooClientConfig::default()).expect("client"),
            true,
        ));

        let (status, _) = send_request(&app, Method::GET, "/health").await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn reboot_returns_ok_when_pixoo_accepts() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(MockMethod::POST).path("/post");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_system_app(system_state(
            PixooClient::new(server.base_url(), PixooClientConfig::default()).expect("client"),
            false,
        ));

        let req = Request::builder()
            .method(Method::POST)
            .uri("/reboot")
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(req).await.expect("response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn reboot_reports_unhealthy_when_pixoo_fails() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(MockMethod::POST).path("/post");
            then.status(200).body(r#"{"error_code":1}"#);
        });

        let app = build_system_app(system_state(
            PixooClient::new(server.base_url(), PixooClientConfig::default()).expect("client"),
            false,
        ));

        let req = Request::builder()
            .method(Method::POST)
            .uri("/reboot")
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(req).await.expect("response");

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
