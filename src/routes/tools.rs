use axum::extract::{Extension, Json, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use pixoo_bridge::pixoo::PixooCommand;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::sync::Arc;
use tracing::error;

use crate::state::AppState;

pub fn mount_tool_routes(router: Router) -> Router {
    router
        .route("/tools/timer/start", post(timer_start))
        .route("/tools/timer/stop", post(timer_stop))
        .route("/tools/stopwatch/{action}", post(stopwatch))
        .route("/tools/scoreboard", post(scoreboard))
        .route("/tools/soundmeter/{action}", post(soundmeter))
}

#[derive(Debug, Deserialize)]
struct TimerRequest {
    minute: u32,
    second: u32,
}

#[derive(Debug, Deserialize)]
struct ScoreboardRequest {
    blue_score: u16,
    red_score: u16,
}

async fn timer_start(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<TimerRequest>,
) -> Response {
    let mut args = Map::new();
    args.insert("Minute".to_string(), Value::from(payload.minute));
    args.insert("Second".to_string(), Value::from(payload.second));
    args.insert("Status".to_string(), Value::from(1));

    dispatch_command(&state, PixooCommand::ToolsTimer, args).await
}

async fn timer_stop(Extension(state): Extension<Arc<AppState>>) -> Response {
    let mut args = Map::new();
    args.insert("Status".to_string(), Value::from(0));

    dispatch_command(&state, PixooCommand::ToolsTimer, args).await
}

async fn stopwatch(
    Path(action): Path<String>,
    Extension(state): Extension<Arc<AppState>>,
) -> Response {
    let status = match action.as_str() {
        "start" => 1,
        "stop" => 0,
        "reset" => 2,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid stopwatch action",
                    "allowed": ["start", "stop", "reset"],
                })),
            )
                .into_response()
        }
    };

    let mut args = Map::new();
    args.insert("Status".to_string(), Value::from(status));

    dispatch_command(&state, PixooCommand::ToolsStopwatch, args).await
}

async fn scoreboard(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<ScoreboardRequest>,
) -> Response {
    if payload.blue_score > 999 || payload.red_score > 999 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "scores must be 0..=999" })),
        )
            .into_response();
    }

    let mut args = Map::new();
    args.insert("BlueScore".to_string(), Value::from(payload.blue_score));
    args.insert("RedScore".to_string(), Value::from(payload.red_score));

    dispatch_command(&state, PixooCommand::ToolsScoreboard, args).await
}

async fn soundmeter(
    Path(action): Path<String>,
    Extension(state): Extension<Arc<AppState>>,
) -> Response {
    let status = match action.as_str() {
        "start" => 1,
        "stop" => 0,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid soundmeter action",
                    "allowed": ["start", "stop"],
                })),
            )
                .into_response()
        }
    };

    let mut args = Map::new();
    args.insert("NoiseStatus".to_string(), Value::from(status));

    dispatch_command(&state, PixooCommand::ToolsSoundMeter, args).await
}

async fn dispatch_command(
    state: &AppState,
    command: PixooCommand,
    args: Map<String, Value>,
) -> Response {
    let client = match state.pixoo_client.clone() {
        Some(client) => client,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "error": "Pixoo client unavailable" })),
            )
                .into_response()
        }
    };

    match client.send_command(command.clone(), args).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => {
            error!(command = %command, error = ?err, "Pixoo tool command failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "error": "Pixoo tool command failed" })),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::routes::mount_tool_routes;
    use crate::state::AppState;
    use axum::body::{to_bytes, Body};
    use axum::extract::Extension;
    use axum::http::{Method, Request, StatusCode};
    use axum::Router;
    use httpmock::{Method as MockMethod, MockServer};
    use pixoo_bridge::pixoo::PixooClient;
    use serde_json::json;
    use std::sync::Arc;
    use tower::ServiceExt;

    fn build_tool_app(state: Arc<AppState>) -> Router {
        mount_tool_routes(Router::new()).layer(Extension(state))
    }

    async fn send_json_request(
        app: &Router,
        method: Method,
        uri: &str,
        body: Option<serde_json::Value>,
    ) -> (StatusCode, String) {
        let builder = Request::builder().method(method).uri(uri);
        let builder = if body.is_some() {
            builder.header("content-type", "application/json")
        } else {
            builder
        };
        let req = builder
            .body(match body {
                Some(value) => Body::from(value.to_string()),
                None => Body::empty(),
            })
            .unwrap();

        let response = app.clone().oneshot(req).await.unwrap();
        let status = response.status();
        let body_bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap_or_default();
        (status, String::from_utf8_lossy(&body_bytes).to_string())
    }

    fn tool_state_with_client(base_url: &str) -> Arc<AppState> {
        let client = PixooClient::new(base_url).expect("client");
        Arc::new(AppState {
            health_forward: false,
            pixoo_client: Some(client),
        })
    }

    #[tokio::test]
    async fn timer_start_succeeds() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(MockMethod::POST).path("/post");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_tool_app(tool_state_with_client(&server.base_url()));
        let (status, body) = send_json_request(
            &app,
            Method::POST,
            "/tools/timer/start",
            Some(json!({"minute": 1, "second": 0})),
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        assert!(body.is_empty());
    }

    #[tokio::test]
    async fn timer_stop_succeeds() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(MockMethod::POST).path("/post");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_tool_app(tool_state_with_client(&server.base_url()));
        let (status, body) = send_json_request(&app, Method::POST, "/tools/timer/stop", None).await;

        assert_eq!(status, StatusCode::OK);
        assert!(body.is_empty());
    }

    #[tokio::test]
    async fn timer_stop_missing_client() {
        let state = Arc::new(AppState {
            health_forward: false,
            pixoo_client: None,
        });
        let app = build_tool_app(state);

        let (status, body) = send_json_request(&app, Method::POST, "/tools/timer/stop", None).await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert!(body.contains("Pixoo client unavailable"));
    }

    #[tokio::test]
    async fn stopwatch_invalid_action() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(MockMethod::POST).path("/post");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_tool_app(tool_state_with_client(&server.base_url()));
        let (status, body) =
            send_json_request(&app, Method::POST, "/tools/stopwatch/fly", None).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(body.contains("invalid stopwatch action"));
    }

    #[tokio::test]
    async fn scoreboard_rejects_out_of_range_scores() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(MockMethod::POST).path("/post");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_tool_app(tool_state_with_client(&server.base_url()));
        let (status, body) = send_json_request(
            &app,
            Method::POST,
            "/tools/scoreboard",
            Some(json!({"blue_score": 1000, "red_score": 0})),
        )
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(body.contains("scores must be 0..=999"));
    }

    #[tokio::test]
    async fn scoreboard_pixoo_error_returns_service_unavailable() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(MockMethod::POST).path("/post");
            then.status(200).body(r#"{"error_code":1}"#);
        });

        let app = build_tool_app(tool_state_with_client(&server.base_url()));
        let (status, body) = send_json_request(
            &app,
            Method::POST,
            "/tools/scoreboard",
            Some(json!({"blue_score": 100, "red_score": 90})),
        )
        .await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert!(body.contains("Pixoo tool command failed"));
    }

    #[tokio::test]
    async fn scoreboard_accepts_valid_scores() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(MockMethod::POST).path("/post");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_tool_app(tool_state_with_client(&server.base_url()));
        let (status, body) = send_json_request(
            &app,
            Method::POST,
            "/tools/scoreboard",
            Some(json!({"blue_score": 12, "red_score": 9})),
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        assert!(body.is_empty());
    }

    #[tokio::test]
    async fn soundmeter_start_succeeds() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(MockMethod::POST).path("/post");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_tool_app(tool_state_with_client(&server.base_url()));
        let (status, _) =
            send_json_request(&app, Method::POST, "/tools/soundmeter/start", None).await;

        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn soundmeter_stop_succeeds() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(MockMethod::POST).path("/post");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_tool_app(tool_state_with_client(&server.base_url()));
        let (status, _) =
            send_json_request(&app, Method::POST, "/tools/soundmeter/stop", None).await;

        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn stopwatch_stop_succeeds() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(MockMethod::POST).path("/post");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_tool_app(tool_state_with_client(&server.base_url()));
        let (status, _) =
            send_json_request(&app, Method::POST, "/tools/stopwatch/stop", None).await;

        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn stopwatch_reset_succeeds() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(MockMethod::POST).path("/post");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_tool_app(tool_state_with_client(&server.base_url()));
        let (status, _) =
            send_json_request(&app, Method::POST, "/tools/stopwatch/reset", None).await;

        assert_eq!(status, StatusCode::OK);
    }
}
