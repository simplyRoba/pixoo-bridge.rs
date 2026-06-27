use crate::pixoo::fields::request as req;
use crate::pixoo::PixooCommand;
use axum::extract::State;
use axum::response::Response;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::str::FromStr;
use std::sync::Arc;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use validator::Validate;

use super::common::{dispatch_pixoo_command, PathParam, ValidatedJson, ValidatedPath};
use crate::pixoo::error::PixooHttpErrorResponse;

use crate::state::AppState;

pub fn tool_router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(timer_start))
        .routes(routes!(timer_stop))
        .routes(routes!(stopwatch))
        .routes(routes!(scoreboard))
        .routes(routes!(soundmeter))
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
struct TimerRequest {
    #[validate(range(min = 0, max = 59))]
    minute: u32,
    #[validate(range(min = 0, max = 59))]
    second: u32,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
struct ScoreboardRequest {
    #[validate(range(min = 0, max = 999))]
    blue_score: u16,
    #[validate(range(min = 0, max = 999))]
    red_score: u16,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
enum StopwatchAction {
    Start,
    Stop,
    Reset,
}

impl StopwatchAction {
    fn status(&self) -> u8 {
        match self {
            Self::Start => 1,
            Self::Stop => 0,
            Self::Reset => 2,
        }
    }
}

impl PathParam for StopwatchAction {
    fn allowed_values() -> &'static [&'static str] {
        &["start", "stop", "reset"]
    }
}

impl FromStr for StopwatchAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "start" => Ok(Self::Start),
            "stop" => Ok(Self::Stop),
            "reset" => Ok(Self::Reset),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
enum SoundmeterAction {
    Start,
    Stop,
}

impl SoundmeterAction {
    fn status(&self) -> u8 {
        match self {
            Self::Start => 1,
            Self::Stop => 0,
        }
    }
}

impl PathParam for SoundmeterAction {
    fn allowed_values() -> &'static [&'static str] {
        &["start", "stop"]
    }
}

impl FromStr for SoundmeterAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "start" => Ok(Self::Start),
            "stop" => Ok(Self::Stop),
            _ => Err(()),
        }
    }
}

#[utoipa::path(
    post,
    path = "/tools/timer/start",
    tag = "tools",
    request_body = TimerRequest,
    responses(
        (status = 200, description = "Timer started"),
        (status = 400, description = "Invalid timer values", body = PixooHttpErrorResponse),
        (status = 502, description = "Pixoo device unreachable", body = PixooHttpErrorResponse),
        (status = 503, description = "Pixoo device reported an error", body = PixooHttpErrorResponse),
        (status = 504, description = "Pixoo device timed out", body = PixooHttpErrorResponse)
    )
)]
#[tracing::instrument(skip(state, payload))]
async fn timer_start(
    State(state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<TimerRequest>,
) -> Response {
    let mut args = Map::new();
    args.insert(req::MINUTE.to_string(), Value::from(payload.minute));
    args.insert(req::SECOND.to_string(), Value::from(payload.second));
    args.insert(req::STATUS.to_string(), Value::from(1));

    dispatch_pixoo_command(&state, PixooCommand::ToolsTimer, args).await
}

#[utoipa::path(
    post,
    path = "/tools/timer/stop",
    tag = "tools",
    responses(
        (status = 200, description = "Timer stopped"),
        (status = 502, description = "Pixoo device unreachable", body = PixooHttpErrorResponse),
        (status = 503, description = "Pixoo device reported an error", body = PixooHttpErrorResponse),
        (status = 504, description = "Pixoo device timed out", body = PixooHttpErrorResponse)
    )
)]
#[tracing::instrument(skip(state))]
async fn timer_stop(State(state): State<Arc<AppState>>) -> Response {
    let mut args = Map::new();
    args.insert(req::STATUS.to_string(), Value::from(0));

    dispatch_pixoo_command(&state, PixooCommand::ToolsTimer, args).await
}

#[utoipa::path(
    post,
    path = "/tools/stopwatch/{action}",
    tag = "tools",
    params(("action" = String, Path, description = "One of: start, stop, reset")),
    responses(
        (status = 200, description = "Stopwatch action applied"),
        (status = 400, description = "Unsupported action", body = PixooHttpErrorResponse),
        (status = 502, description = "Pixoo device unreachable", body = PixooHttpErrorResponse),
        (status = 503, description = "Pixoo device reported an error", body = PixooHttpErrorResponse),
        (status = 504, description = "Pixoo device timed out", body = PixooHttpErrorResponse)
    )
)]
#[tracing::instrument(skip(state))]
async fn stopwatch(
    State(state): State<Arc<AppState>>,
    ValidatedPath(action): ValidatedPath<StopwatchAction>,
) -> Response {
    let mut args = Map::new();
    args.insert(req::STATUS.to_string(), Value::from(action.status()));

    dispatch_pixoo_command(&state, PixooCommand::ToolsStopwatch, args).await
}

#[utoipa::path(
    post,
    path = "/tools/scoreboard",
    tag = "tools",
    request_body = ScoreboardRequest,
    responses(
        (status = 200, description = "Scoreboard updated"),
        (status = 400, description = "Invalid scores", body = PixooHttpErrorResponse),
        (status = 502, description = "Pixoo device unreachable", body = PixooHttpErrorResponse),
        (status = 503, description = "Pixoo device reported an error", body = PixooHttpErrorResponse),
        (status = 504, description = "Pixoo device timed out", body = PixooHttpErrorResponse)
    )
)]
#[tracing::instrument(skip(state, payload))]
async fn scoreboard(
    State(state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<ScoreboardRequest>,
) -> Response {
    let mut args = Map::new();
    args.insert(req::BLUE_SCORE.to_string(), Value::from(payload.blue_score));
    args.insert(req::RED_SCORE.to_string(), Value::from(payload.red_score));

    dispatch_pixoo_command(&state, PixooCommand::ToolsScoreboard, args).await
}

#[utoipa::path(
    post,
    path = "/tools/soundmeter/{action}",
    tag = "tools",
    params(("action" = String, Path, description = "One of: start, stop")),
    responses(
        (status = 200, description = "Sound meter action applied"),
        (status = 400, description = "Unsupported action", body = PixooHttpErrorResponse),
        (status = 502, description = "Pixoo device unreachable", body = PixooHttpErrorResponse),
        (status = 503, description = "Pixoo device reported an error", body = PixooHttpErrorResponse),
        (status = 504, description = "Pixoo device timed out", body = PixooHttpErrorResponse)
    )
)]
#[tracing::instrument(skip(state))]
async fn soundmeter(
    State(state): State<Arc<AppState>>,
    ValidatedPath(action): ValidatedPath<SoundmeterAction>,
) -> Response {
    let mut args = Map::new();
    args.insert(req::NOISE_STATUS.to_string(), Value::from(action.status()));

    dispatch_pixoo_command(&state, PixooCommand::ToolsSoundMeter, args).await
}

#[cfg(test)]
mod tests {
    use super::tool_router;
    use crate::pixoo::{PixooClient, PixooClientConfig};
    use crate::routes::common::testing::send_json_request;
    use crate::state::AppState;
    use axum::http::{Method, StatusCode};
    use axum::Router;
    use httpmock::{Method as MockMethod, MockServer};
    use serde_json::{json, Value};
    use std::sync::Arc;

    fn build_tool_app(state: Arc<AppState>) -> Router {
        let (router, _api) = tool_router().with_state(state).split_for_parts();
        router
    }

    fn assert_validation_failed(body: &str) -> Value {
        let parsed: Value = serde_json::from_str(body).unwrap();
        assert_eq!(parsed["error_kind"], "validation");
        parsed
    }

    fn tool_state_with_client(base_url: &str) -> Arc<AppState> {
        let client = PixooClient::new(base_url, PixooClientConfig::default()).expect("client");
        Arc::new(AppState::with_client(client))
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
    async fn timer_start_rejects_invalid_minute() {
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
            Some(json!({"minute": 60, "second": 0})),
        )
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        let body_json = assert_validation_failed(&body);
        assert!(body_json["details"]["minute"].is_array());
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
        let body_json = assert_validation_failed(&body);
        assert_eq!(body_json["details"]["action"]["provided"], "fly");
        let allowed = body_json["details"]["action"]["allowed"]
            .as_array()
            .unwrap();
        assert_eq!(allowed.len(), 3);
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
        let body_json = assert_validation_failed(&body);
        assert!(body_json["details"]["blue_score"].is_array());
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
        let json_body: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json_body["error_kind"], "device-error");
        assert_eq!(json_body["error_status"], 503);
        assert_eq!(json_body["details"]["error_code"], 1);
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
    async fn soundmeter_invalid_action() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(MockMethod::POST).path("/post");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_tool_app(tool_state_with_client(&server.base_url()));
        let (status, body) =
            send_json_request(&app, Method::POST, "/tools/soundmeter/fly", None).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        let body_json = assert_validation_failed(&body);
        assert_eq!(body_json["details"]["action"]["provided"], "fly");
        let allowed = body_json["details"]["action"]["allowed"]
            .as_array()
            .unwrap();
        assert_eq!(allowed.len(), 2);
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
