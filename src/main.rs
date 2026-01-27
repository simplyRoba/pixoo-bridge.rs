use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde_json::json;
use std::{env, net::SocketAddr};
use tracing::info;
use tracing_subscriber::fmt::init;

use pixoo_bridge::pixoo::PixooClient;

#[derive(Clone)]
struct AppState {
    health_forward: bool,
    pixoo_client: Option<PixooClient>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init();

    let health_forward = read_bool_env("PIXOO_BRIDGE_HEALTH_FORWARD", true);
    let pixoo_client = env::var("PIXOO_BASE_URL")
        .ok()
        .and_then(|base_url| PixooClient::new(base_url).ok());
    let state = AppState {
        health_forward,
        pixoo_client,
    };
    let app = build_app(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Pixoo bridge listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> &'static str {
    "Hello World from Pixoo Bridge!"
}

fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    if !state.health_forward {
        return (StatusCode::OK, Json(json!({ "status": "ok" })));
    }

    let client = match state.pixoo_client {
        Some(client) => client,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "status": "unhealthy" })),
            );
        }
    };

    match client.health_check().await {
        Ok(()) => (StatusCode::OK, Json(json!({ "status": "ok" }))),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "status": "unhealthy" })),
        ),
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use axum::http::Request;
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use std::sync::{Mutex, OnceLock};
    use tower::util::ServiceExt;

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static ENV_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_MUTEX
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("lock")
    }

    #[tokio::test]
    async fn health_ok_when_forwarding_disabled() {
        let state = AppState {
            health_forward: false,
            pixoo_client: None,
        };
        let app = build_app(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        assert_eq!(body, r#"{"status":"ok"}"#);
    }

    #[tokio::test]
    async fn health_ok_when_pixoo_healthy() {
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(GET).path("/get");
            then.status(200);
        });

        let client = PixooClient::new(server.base_url()).expect("client");
        let state = AppState {
            health_forward: true,
            pixoo_client: Some(client),
        };
        let app = build_app(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        assert_eq!(body, r#"{"status":"ok"}"#);
        mock.assert();
    }

    #[tokio::test]
    async fn health_forwarding_enabled_by_default() {
        let _guard = env_lock();
        let original = env::var("PIXOO_BRIDGE_HEALTH_FORWARD").ok();
        unsafe {
            env::remove_var("PIXOO_BRIDGE_HEALTH_FORWARD");
        }

        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(GET).path("/get");
            then.status(200);
        });

        let state = AppState {
            health_forward: read_bool_env("PIXOO_BRIDGE_HEALTH_FORWARD", true),
            pixoo_client: Some(PixooClient::new(server.base_url()).expect("client")),
        };
        let app = build_app(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        mock.assert();

        match original {
            Some(value) => unsafe {
                env::set_var("PIXOO_BRIDGE_HEALTH_FORWARD", value);
            },
            None => unsafe {
                env::remove_var("PIXOO_BRIDGE_HEALTH_FORWARD");
            },
        }
    }

    #[tokio::test]
    async fn health_reports_unhealthy_on_pixoo_failure() {
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(GET).path("/get");
            then.status(500);
        });

        let client = PixooClient::new(server.base_url()).expect("client");
        let state = AppState {
            health_forward: true,
            pixoo_client: Some(client),
        };
        let app = build_app(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        mock.assert_calls(3);
    }
}
