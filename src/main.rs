mod config;
mod openapi;
mod pixels;
mod pixoo;
mod remote;
mod request_tracing;
mod routes;
mod state;

use axum::{
    body::Body,
    http::Request,
    middleware::{from_fn, Next},
    response::{Redirect, Response},
    routing::get,
    Router,
};
use std::{env, net::SocketAddr, sync::Arc, time::Instant};
use tokio::signal;
use tracing::{debug, error, info, warn};
use tracing_subscriber::filter::LevelFilter;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

use config::{AppConfig, ConfigSource, EnvConfigSource};
use openapi::ApiDoc;
use pixoo::PixooClient;
use remote::{RemoteFetchConfig, RemoteFetcher};
use request_tracing::RequestId;
use routes::build_router;
use state::AppState;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::{Config, SwaggerUi};

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
    let remote_fetcher = RemoteFetcher::new(RemoteFetchConfig::new(
        config.remote_timeout,
        config.max_image_size,
    ))?;
    let state = Arc::new(AppState {
        health_forward: config.health_forward,
        pixoo_client,
        animation_speed_factor: config.animation_speed_factor,
        max_image_size: config.max_image_size,
        remote_fetcher,
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
        remote_timeout = ?config.remote_timeout,
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
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(build_router())
        .split_for_parts();

    router
        .merge(
            SwaggerUi::new("/docs")
                .url("/api-docs/openapi.json", api)
                .config(Config::from("/api-docs/openapi.json").validator_url("none")),
        )
        .route("/", get(|| async { Redirect::permanent("/docs") }))
        .fallback(fallback_not_found)
        .layer(CorsLayer::permissive())
        .layer(from_fn(access_log))
        .layer(from_fn(request_tracing::propagate))
        .with_state(state)
}

async fn fallback_not_found() -> Response {
    routes::not_found()
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
    use crate::config::testing::MockConfig;
    use crate::pixoo::{PixooClient, PixooClientConfig};
    use crate::state::AppState;
    use axum::body::Body;
    use axum::http::{Method, Request, StatusCode};
    use httpmock::{Method as MockMethod, MockServer};
    use std::sync::Arc;
    use tower::util::ServiceExt;

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

    #[tokio::test]
    async fn error_path_unreachable_device_returns_502_with_request_id() {
        let client = PixooClient::new(
            "http://127.0.0.1:1".to_string(),
            PixooClientConfig::default(),
        )
        .expect("client");
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

        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);

        let request_id = response
            .headers()
            .get("X-Request-Id")
            .expect("X-Request-Id header");
        assert!(!request_id.to_str().unwrap().is_empty());

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error_kind"], "unreachable");
        assert_eq!(json["error_status"], 502);
        assert!(!json["message"].as_str().unwrap().is_empty());
    }

    #[tokio::test]
    async fn error_path_device_error_returns_503_with_request_id() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(MockMethod::POST).path("/post");
            then.status(200).body(r#"{"error_code":1}"#);
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

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

        let request_id = response
            .headers()
            .get("X-Request-Id")
            .expect("X-Request-Id header");
        assert!(!request_id.to_str().unwrap().is_empty());

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error_kind"], "device-error");
        assert_eq!(json["error_status"], 503);
        assert_eq!(json["details"]["error_code"], 1);
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

    #[tokio::test]
    async fn openapi_spec_endpoint_returns_document_with_routes() {
        let server = MockServer::start_async().await;
        let client =
            PixooClient::new(server.base_url(), PixooClientConfig::default()).expect("client");
        let app = build_app(Arc::new(AppState::with_client(client)));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api-docs/openapi.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let doc: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(doc["openapi"].as_str().unwrap().chars().next(), Some('3'));
        assert_eq!(doc["info"]["title"], "Pixoo Bridge");
        let paths = doc["paths"].as_object().expect("paths object");
        assert!(paths.contains_key("/draw/fill"));
        assert!(paths.contains_key("/manage/settings"));
        assert!(paths.contains_key("/health"));
        assert!(paths.contains_key("/tools/stopwatch/{action}"));
        assert!(doc["paths"]["/health"].get("get").is_some());
        assert!(doc["paths"]["/draw/fill"].get("post").is_some());

        // Schema component still present; no divergent one-off schemas.
        let schemas = doc["components"]["schemas"].as_object().expect("schemas");
        assert!(schemas.contains_key("PixooHttpErrorResponse"));
        assert!(!schemas.contains_key("ValidationErrorBody"));
        assert!(!schemas.contains_key("PayloadTooLargeBody"));

        // All reusable response components are registered and each one points
        // back to the canonical envelope schema.
        let resp_components = doc["components"]["responses"]
            .as_object()
            .expect("response components");
        for name in [
            "ValidationErrorResponse",
            "PayloadTooLargeResponse",
            "InternalErrorResponse",
            "DeviceUnreachableResponse",
            "DeviceErrorResponse",
            "DeviceTimeoutResponse",
        ] {
            assert!(
                resp_components.contains_key(name),
                "missing response component: {name}"
            );
            let schema_ref = resp_components[name]["content"]["application/json"]["schema"]["$ref"]
                .as_str()
                .unwrap_or_default();
            assert_eq!(
                schema_ref, "#/components/schemas/PixooHttpErrorResponse",
                "{name} must reference the canonical envelope schema"
            );
        }

        // Path-level responses now $ref the response components (not inline schema).
        let fill_responses = &doc["paths"]["/draw/fill"]["post"]["responses"];
        for status in ["400", "500", "502", "503", "504"] {
            let ref_val = fill_responses[status]["$ref"].as_str().unwrap_or_default();
            assert!(
                ref_val.starts_with("#/components/responses/"),
                "status {status} must $ref a response component, got: {ref_val:?}"
            );
        }
    }

    #[tokio::test]
    async fn root_path_redirects_to_docs() {
        let server = MockServer::start_async().await;
        let client =
            PixooClient::new(server.base_url(), PixooClientConfig::default()).expect("client");
        let app = build_app(Arc::new(AppState::with_client(client)));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
        assert_eq!(
            response
                .headers()
                .get("location")
                .unwrap()
                .to_str()
                .unwrap(),
            "/docs"
        );
    }

    #[tokio::test]
    async fn swagger_ui_is_served_at_docs() {
        let server = MockServer::start_async().await;
        let client =
            PixooClient::new(server.base_url(), PixooClientConfig::default()).expect("client");
        let app = build_app(Arc::new(AppState::with_client(client)));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/docs/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn undefined_route_returns_json_404() {
        let server = MockServer::start_async().await;
        let client =
            PixooClient::new(server.base_url(), PixooClientConfig::default()).expect("client");
        let app = build_app(Arc::new(AppState::with_client(client)));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error_status"], 404);
        assert_eq!(json["error_kind"], "not-found");
        assert!(json["message"].is_string());
        assert!(
            json.get("details").is_none(),
            "not-found body must omit details"
        );
    }
}
