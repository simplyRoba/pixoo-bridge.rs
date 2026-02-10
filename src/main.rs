mod config;
mod pixels;
mod pixoo;
mod request_tracing;
mod routes;
mod state;

use axum::{
    body::Body,
    http::Request,
    middleware::{from_fn, Next},
    response::Response,
    Router,
};
use std::{env, net::SocketAddr, sync::Arc, time::Instant};
use tokio::signal;
use tracing::{debug, error, info, warn};
use tracing_subscriber::filter::LevelFilter;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

use config::{AppConfig, ConfigSource, EnvConfigSource};
use pixoo::PixooClient;
use request_tracing::RequestId;
use routes::mount_all_routes;
use state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (max_level, invalid_level) = resolve_log_level(&EnvConfigSource);
    tracing_subscriber::fmt().with_max_level(max_level).init();

    if let Some(raw) = invalid_level {
        warn!(invalid_level = %raw, "Invalid PIXOO_BRIDGE_LOG_LEVEL, defaulting to info");
    }

    let config = match AppConfig::load() {
        Ok(config) => config,
        Err(err) => {
            error!(error = %err, "Configuration error");
            return Err(err.into());
        }
    };
    let pixoo_client = PixooClient::new(config.pixoo_base_url.clone(), config.pixoo_client)?;
    let state = Arc::new(AppState {
        health_forward: config.health_forward,
        pixoo_client,
        animation_speed_factor: config.animation_speed_factor,
        max_image_size: config.max_image_size,
    });
    let app = build_app(state.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.listener_port));
    info!(
        version = APP_VERSION,
        address = %addr,
        listener_port = config.listener_port,
        log_level = ?max_level,
        pixoo_base_url = %config.pixoo_base_url,
        pixoo_client = true,
        health_forward = config.health_forward,
        animation_speed_factor = config.animation_speed_factor,
        max_image_size = config.max_image_size,
        "Pixoo bridge configuration loaded"
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Graceful shutdown complete");
    Ok(())
}

fn build_app(state: Arc<AppState>) -> Router {
    let app = mount_all_routes(Router::new());

    app.layer(from_fn(access_log))
        .layer(from_fn(request_tracing::propagate))
        .with_state(state)
}

async fn access_log(req: Request<Body>, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let start = Instant::now();
    let request_id = req
        .extensions()
        .get::<RequestId>()
        .cloned()
        .unwrap_or_default();
    let response = next.run(req).await;
    let latency = start.elapsed();
    let status = response.status();
    debug!(method=%method, path=%path, status=%status, latency=?latency, request_id=%request_id, "access log");
    response
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {
            info!("Received SIGINT, starting graceful shutdown");
        }
        () = terminate => {
            info!("Received SIGTERM, starting graceful shutdown");
        }
    }
}

fn resolve_log_level(source: &impl ConfigSource) -> (LevelFilter, Option<String>) {
    let raw = source
        .get("PIXOO_BRIDGE_LOG_LEVEL")
        .unwrap_or_else(|| "info".to_string());
    match raw.parse::<LevelFilter>() {
        Ok(level) => (level, None),
        Err(_) => (LevelFilter::INFO, Some(raw)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pixoo::{PixooClient, PixooClientConfig};
    use crate::state::AppState;
    use axum::body::Body;
    use axum::http::{Method, Request, StatusCode};
    use httpmock::{Method as MockMethod, MockServer};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tower::util::ServiceExt;

    struct MockConfig(HashMap<&'static str, &'static str>);

    impl MockConfig {
        fn new() -> Self {
            Self(HashMap::new())
        }

        fn with(mut self, key: &'static str, value: &'static str) -> Self {
            self.0.insert(key, value);
            self
        }
    }

    impl ConfigSource for MockConfig {
        fn get(&self, key: &str) -> Option<String> {
            self.0.get(key).map(|s| (*s).to_string())
        }
    }

    #[tokio::test]
    async fn integration_build_app_includes_tool_routes() {
        let server = MockServer::start_async().await;
        let _mock = server.mock(|when, then| {
            when.method(MockMethod::POST).path("/post");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let client =
            PixooClient::new(server.base_url(), PixooClientConfig::default()).expect("client");
        let app = build_app(Arc::new(AppState::with_client(client)));

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

    #[tokio::test]
    async fn integration_build_app_includes_system_routes() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(MockMethod::GET).path("/get");
            then.status(200);
        });

        let client =
            PixooClient::new(server.base_url(), PixooClientConfig::default()).expect("client");
        let mut state = AppState::with_client(client);
        state.health_forward = true;
        let app = build_app(Arc::new(state));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn request_id_middleware_inserts_header() {
        let server = MockServer::start_async().await;
        let client =
            PixooClient::new(server.base_url(), PixooClientConfig::default()).expect("client");
        let app = build_app(Arc::new(AppState::with_client(client)));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        let header = response
            .headers()
            .get("X-Request-Id")
            .expect("request id header present");
        assert!(!header.to_str().unwrap().is_empty());
    }

    #[test]
    fn resolves_log_level_defaults_to_info() {
        let config = MockConfig::new();
        let (level, invalid) = resolve_log_level(&config);
        assert_eq!(level, LevelFilter::INFO);
        assert!(invalid.is_none());
    }

    #[test]
    fn resolves_log_level_from_env() {
        let config = MockConfig::new().with("PIXOO_BRIDGE_LOG_LEVEL", "debug");
        let (level, invalid) = resolve_log_level(&config);
        assert_eq!(level, LevelFilter::DEBUG);
        assert!(invalid.is_none());
    }

    #[test]
    fn resolves_log_level_invalid_falls_back_to_info() {
        let config = MockConfig::new().with("PIXOO_BRIDGE_LOG_LEVEL", "not-a-level");
        let (level, invalid) = resolve_log_level(&config);
        assert_eq!(level, LevelFilter::INFO);
        assert_eq!(invalid, Some("not-a-level".to_string()));
    }
}
