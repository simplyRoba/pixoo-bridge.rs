use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use pixoo_bridge::pixoo::PixooCommand;
use serde_json::{json, Map, Value};
use std::sync::Arc;
use tracing::{debug, error};

use crate::state::AppState;

pub fn mount_system_routes(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router
        .route("/health", get(health))
        .route("/reboot", post(reboot))
}

async fn health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if !state.health_forward {
        return (StatusCode::OK, Json(json!({ "status": "ok" })));
    }

    let client = match state.pixoo_client.clone() {
        Some(client) => client,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "status": "unhealthy" })),
            );
        }
    };

    match client.health_check().await {
        Ok(()) => {
            debug!("Pixoo health check succeeded");
            (StatusCode::OK, Json(json!({ "status": "ok" })))
        }
        Err(err) => {
            error!(error = ?err, status = %StatusCode::SERVICE_UNAVAILABLE, "Pixoo health check failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "status": "unhealthy" })),
            )
        }
    }
}

async fn reboot(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let client = match state.pixoo_client.clone() {
        Some(client) => client,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "error": "Pixoo client unavailable" })),
            )
                .into_response();
        }
    };

    match client
        .send_command(PixooCommand::SystemReboot, Map::<String, Value>::new())
        .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => {
            error!(error = ?err, "Pixoo reboot command failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "error": "Pixoo reboot failed" })),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use axum::http::{Method, Request, StatusCode};
    use axum::Router;
    use httpmock::{Method as MockMethod, MockServer};
    use pixoo_bridge::pixoo::PixooClient;
    use std::env;
    use std::sync::{Arc, Mutex, OnceLock};
    use tower::ServiceExt;

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static ENV_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_MUTEX
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("lock")
    }

    fn with_env_var<T>(key: &str, value: Option<&str>, f: impl FnOnce() -> T) -> T {
        let _guard = env_lock();
        let original = env::var(key).ok();
        match value {
            Some(v) => env::set_var(key, v),
            None => env::remove_var(key),
        }
        let result = f();
        match original {
            Some(v) => env::set_var(key, v),
            None => env::remove_var(key),
        }
        result
    }

    fn read_bool_env(key: &str, default: bool) -> bool {
        match env::var(key) {
            Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
                "1" | "true" | "yes" | "on" => true,
                "0" | "false" | "no" | "off" => false,
                _ => default,
            },
            Err(_) => default,
        }
    }

    fn build_system_app(state: Arc<AppState>) -> Router {
        mount_system_routes(Router::new()).with_state(state)
    }

    fn system_state(client: Option<PixooClient>, health_forward: bool) -> Arc<AppState> {
        Arc::new(AppState {
            health_forward,
            pixoo_client: client,
        })
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
        let app = build_system_app(system_state(None, false));

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
            Some(PixooClient::new(server.base_url()).expect("client")),
            true,
        ));

        let (status, body) = send_request(&app, Method::GET, "/health").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, r#"{"status":"ok"}"#);
    }

    #[tokio::test]
    async fn health_forwarding_enabled_by_default() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(MockMethod::GET).path("/get");
            then.status(200);
        });

        let health_forward = with_env_var("PIXOO_BRIDGE_HEALTH_FORWARD", None, || {
            read_bool_env("PIXOO_BRIDGE_HEALTH_FORWARD", true)
        });
        let app = build_system_app(system_state(
            Some(PixooClient::new(server.base_url()).expect("client")),
            health_forward,
        ));

        let (status, _) = send_request(&app, Method::GET, "/health").await;

        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn health_reports_unhealthy_on_pixoo_failure() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(MockMethod::GET).path("/get");
            then.status(500);
        });

        let app = build_system_app(system_state(
            Some(PixooClient::new(server.base_url()).expect("client")),
            true,
        ));

        let (status, _) = send_request(&app, Method::GET, "/health").await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn reboot_returns_no_content_when_pixoo_accepts() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(MockMethod::POST).path("/post");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_system_app(system_state(
            Some(PixooClient::new(server.base_url()).expect("client")),
            false,
        ));

        let req = Request::builder()
            .method(Method::POST)
            .uri("/reboot")
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(req).await.expect("response");

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn reboot_reports_unhealthy_when_pixoo_fails() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(MockMethod::POST).path("/post");
            then.status(200).body(r#"{"error_code":1}"#);
        });

        let app = build_system_app(system_state(
            Some(PixooClient::new(server.base_url()).expect("client")),
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
