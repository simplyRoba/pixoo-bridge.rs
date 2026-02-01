mod routes;
mod state;

use axum::{extract::Extension, routing::get, Router};
use reqwest::Url;
use std::{env, net::SocketAddr, sync::Arc};
use tracing::{info, warn};
use tracing_subscriber::filter::LevelFilter;

const DEFAULT_LISTENER_PORT: u16 = 4000;
const MIN_LISTENER_PORT: u16 = 1024;
const MAX_LISTENER_PORT: u16 = 65535;
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

use pixoo_bridge::pixoo::PixooClient;
use routes::{mount_system_routes, mount_tool_routes};
use state::AppState;

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
    let state = Arc::new(AppState {
        health_forward,
        pixoo_client,
    });
    let has_pixoo_client = state.pixoo_client.is_some();
    let app = build_app(state.clone());

    let listener_port = resolve_listener_port();
    let addr = SocketAddr::from(([0, 0, 0, 0], listener_port));
    info!(
        log_level = ?max_level,
        health_forward,
        pixoo_client = has_pixoo_client,
        sanitized_pixoo_base_url = ?sanitized_base_url,
        listener_port,
        version = APP_VERSION,
        address = %addr,
        "Pixoo bridge configuration loaded"
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn root() -> &'static str {
    "Hello World from Pixoo Bridge!"
}

fn build_app(state: Arc<AppState>) -> Router {
    let app = Router::new().route("/", get(root));
    let app = mount_tool_routes(app);
    mount_system_routes(app).layer(Extension(state))
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

fn resolve_listener_port() -> u16 {
    match env::var("PIXOO_BRIDGE_PORT") {
        Ok(raw) => {
            let value = raw.trim();
            match value.parse::<u16>() {
                Ok(port) if (MIN_LISTENER_PORT..=MAX_LISTENER_PORT).contains(&port) => port,
                _ => {
                    warn!(
                        provided = %value,
                        min = MIN_LISTENER_PORT,
                        max = MAX_LISTENER_PORT,
                        default_port = DEFAULT_LISTENER_PORT,
                        "Invalid PIXOO_BRIDGE_PORT; falling back to default port {}",
                        DEFAULT_LISTENER_PORT
                    );
                    DEFAULT_LISTENER_PORT
                }
            }
        }
        Err(_) => DEFAULT_LISTENER_PORT,
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
    use crate::state::AppState;
    use axum::body::{to_bytes, Body};
    use axum::http::{Method, Request, StatusCode};
    use httpmock::{Method as MockMethod, MockServer};
    use pixoo_bridge::pixoo::PixooClient;
    use std::sync::{Arc, Mutex, OnceLock};
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
        let app = build_app(Arc::new(state));

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
        let _mock = server.mock(|when, then| {
            when.method(MockMethod::GET).path("/get");
            then.status(200);
        });

        let client = PixooClient::new(server.base_url()).expect("client");
        let state = AppState {
            health_forward: true,
            pixoo_client: Some(client),
        };
        let app = build_app(Arc::new(state));

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
    async fn health_forwarding_enabled_by_default() {
        let server = MockServer::start_async().await;
        let _mock = server.mock(|when, then| {
            when.method(MockMethod::GET).path("/get");
            then.status(200);
        });

        let health_forward = with_env_var("PIXOO_BRIDGE_HEALTH_FORWARD", None, || {
            read_bool_env("PIXOO_BRIDGE_HEALTH_FORWARD", true)
        });
        let state = AppState {
            health_forward,
            pixoo_client: Some(PixooClient::new(server.base_url()).expect("client")),
        };
        let app = build_app(Arc::new(state));

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
    }

    #[tokio::test]
    async fn health_reports_unhealthy_on_pixoo_failure() {
        let server = MockServer::start_async().await;
        let _mock = server.mock(|when, then| {
            when.method(MockMethod::GET).path("/get");
            then.status(500);
        });

        let client = PixooClient::new(server.base_url()).expect("client");
        let state = AppState {
            health_forward: true,
            pixoo_client: Some(client),
        };
        let app = build_app(Arc::new(state));

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
    }

    #[tokio::test]
    async fn reboot_returns_no_content_when_pixoo_accepts() {
        let server = MockServer::start_async().await;
        let _mock = server.mock(|when, then| {
            when.method(MockMethod::POST).path("/post");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let client = PixooClient::new(server.base_url()).expect("client");
        let state = AppState {
            health_forward: false,
            pixoo_client: Some(client),
        };
        let app = build_app(Arc::new(state));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/reboot")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn reboot_reports_unhealthy_when_pixoo_fails() {
        let server = MockServer::start_async().await;
        let _mock = server.mock(|when, then| {
            when.method(MockMethod::POST).path("/post");
            then.status(500).body(r#"{"error_code":0}"#);
        });

        let client = PixooClient::new(server.base_url()).expect("client");
        let state = AppState {
            health_forward: false,
            pixoo_client: Some(client),
        };
        let app = build_app(Arc::new(state));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/reboot")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn tools_stopwatch_route_available_via_build_app() {
        let server = MockServer::start_async().await;
        let _mock = server.mock(|when, then| {
            when.method(MockMethod::POST).path("/post");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let client = PixooClient::new(server.base_url()).expect("client");
        let state = AppState {
            health_forward: false,
            pixoo_client: Some(client),
        };
        let app = build_app(Arc::new(state));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/tools/stopwatch/start")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
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

    #[test]
    fn listener_port_defaults_to_4000_when_env_missing() {
        let port = with_env_var("PIXOO_BRIDGE_PORT", None, resolve_listener_port);
        assert_eq!(port, DEFAULT_LISTENER_PORT);
    }

    #[test]
    fn listener_port_uses_custom_override_when_valid() {
        let port = with_env_var("PIXOO_BRIDGE_PORT", Some("5050"), resolve_listener_port);
        assert_eq!(port, 5050);
    }

    #[test]
    fn listener_port_falls_back_on_invalid_values() {
        let port = with_env_var(
            "PIXOO_BRIDGE_PORT",
            Some("not-a-port"),
            resolve_listener_port,
        );
        assert_eq!(port, DEFAULT_LISTENER_PORT);
    }
}
