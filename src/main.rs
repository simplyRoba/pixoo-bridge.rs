use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use reqwest::Url;
use serde_json::json;
use std::{env, net::SocketAddr};
use tracing::{debug, error, info, warn};
use tracing_subscriber::filter::LevelFilter;

use pixoo_bridge::pixoo::PixooClient;

#[derive(Clone)]
struct AppState {
    health_forward: bool,
    pixoo_client: Option<PixooClient>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (max_level, invalid_level) = resolve_log_level();
    tracing_subscriber::fmt().with_max_level(max_level).init();

    if let Some(raw) = invalid_level {
        warn!(invalid_level = %raw, "Invalid PIXOO_BRIDGE_LOG_LEVEL, defaulting to info");
    }

    let health_forward = read_bool_env("PIXOO_BRIDGE_HEALTH_FORWARD", true);
    let base_url = env::var("PIXOO_BASE_URL").ok();
    let pixoo_client = base_url
        .as_deref()
        .and_then(|base| PixooClient::new(base).ok());
    let sanitized_base_url = base_url.as_deref().and_then(sanitize_pixoo_url);
    let state = AppState {
        health_forward,
        pixoo_client,
    };
    let has_pixoo_client = state.pixoo_client.is_some();
    let app = build_app(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!(
        log_level = ?max_level,
        health_forward,
        pixoo_client = has_pixoo_client,
        sanitized_pixoo_base_url = ?sanitized_base_url,
        address = %addr,
        "Pixoo bridge configuration loaded"
    );

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
        Ok(()) => {
            debug!("Pixoo health check succeeded");
            (StatusCode::OK, Json(json!({ "status": "ok" })))
        }
        Err(err) => {
            error!(
                error = ?err,
                status = %StatusCode::SERVICE_UNAVAILABLE,
                "Pixoo health check failed"
            );
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "status": "unhealthy" })),
            )
        }
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

fn resolve_log_level() -> (LevelFilter, Option<String>) {
    let raw = env::var("PIXOO_BRIDGE_LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    match raw.parse::<LevelFilter>() {
        Ok(level) => (level, None),
        Err(_) => (LevelFilter::INFO, Some(raw)),
    }
}

fn sanitize_pixoo_url(value: &str) -> Option<String> {
    let url = Url::parse(value).ok()?;
    let host = url.host_str()?;
    Some(format!("{}://{}", url.scheme(), host))
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

    fn with_env_var<T>(key: &str, value: Option<&str>, f: impl FnOnce() -> T) -> T {
        let _guard = env_lock();
        let original = env::var(key).ok();
        match value {
            Some(v) => unsafe { env::set_var(key, v) },
            None => unsafe { env::remove_var(key) },
        }
        let result = f();
        match original {
            Some(v) => unsafe { env::set_var(key, v) },
            None => unsafe { env::remove_var(key) },
        }
        result
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
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(GET).path("/get");
            then.status(200);
        });

        let health_forward = with_env_var("PIXOO_BRIDGE_HEALTH_FORWARD", None, || {
            read_bool_env("PIXOO_BRIDGE_HEALTH_FORWARD", true)
        });
        let state = AppState {
            health_forward,
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

    #[test]
    fn resolves_log_level_defaults_to_info() {
        let (level, invalid) = with_env_var("PIXOO_BRIDGE_LOG_LEVEL", None, resolve_log_level);
        assert_eq!(level, LevelFilter::INFO);
        assert!(invalid.is_none());
    }

    #[test]
    fn resolves_log_level_from_env() {
        let (level, invalid) =
            with_env_var("PIXOO_BRIDGE_LOG_LEVEL", Some("debug"), resolve_log_level);
        assert_eq!(level, LevelFilter::DEBUG);
        assert!(invalid.is_none());
    }

    #[test]
    fn resolves_log_level_invalid_falls_back_to_info() {
        let (level, invalid) = with_env_var(
            "PIXOO_BRIDGE_LOG_LEVEL",
            Some("not-a-level"),
            resolve_log_level,
        );
        assert_eq!(level, LevelFilter::INFO);
        assert_eq!(invalid, Some("not-a-level".to_string()));
    }
}
