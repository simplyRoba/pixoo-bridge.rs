use crate::pixoo::{map_pixoo_error, PixooCommand};
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::str::FromStr;
use std::sync::Arc;
use tracing::error;
use validator::Validate;

use super::common::{action_validation_error, validation_errors_response};

use crate::state::AppState;

pub fn mount_tool_routes(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router
        .route("/tools/timer/start", post(timer_start))
        .route("/tools/timer/stop", post(timer_stop))
        .route("/tools/stopwatch/{action}", post(stopwatch))
        .route("/tools/scoreboard", post(scoreboard))
        .route("/tools/soundmeter/{action}", post(soundmeter))
}

#[derive(Debug, Deserialize, Validate)]
struct TimerRequest {
    #[validate(range(min = 0, max = 59))]
    minute: u32,
    #[validate(range(min = 0, max = 59))]
    second: u32,
}

#[derive(Debug, Deserialize, Validate)]
struct ScoreboardRequest {
    #[validate(range(min = 0, max = 999))]
    blue_score: u16,
    #[validate(range(min = 0, max = 999))]
    red_score: u16,
}

#[derive(Debug, Deserialize)]
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

    fn allowed_values() -> &'static [&'static str] {
        &["start", "stop", "reset"]
    }
}

impl FromStr for StopwatchAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "start" => Ok(Self::Start),
            "stop" => Ok(Self::Stop),
            "reset" => Ok(Self::Reset),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Deserialize)]
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

    fn allowed_values() -> &'static [&'static str] {
        &["start", "stop"]
    }
}

impl FromStr for SoundmeterAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "start" => Ok(Self::Start),
            "stop" => Ok(Self::Stop),
            _ => Err(()),
        }
    }
}

#[tracing::instrument(skip(state, payload))]
async fn timer_start(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TimerRequest>,
) -> Response {
    if let Err(errors) = payload.validate() {
        return validation_errors_response(&errors);
    }

    let mut args = Map::new();
    args.insert("Minute".to_string(), Value::from(payload.minute));
    args.insert("Second".to_string(), Value::from(payload.second));
    args.insert("Status".to_string(), Value::from(1));

    dispatch_command(&state, PixooCommand::ToolsTimer, args).await
}

#[tracing::instrument(skip(state))]
async fn timer_stop(State(state): State<Arc<AppState>>) -> Response {
    let mut args = Map::new();
    args.insert("Status".to_string(), Value::from(0));

    dispatch_command(&state, PixooCommand::ToolsTimer, args).await
}

#[tracing::instrument(skip(state))]
async fn stopwatch(State(state): State<Arc<AppState>>, Path(action): Path<String>) -> Response {
    let Ok(parsed) = action.parse::<StopwatchAction>() else {
        return action_validation_error(&action, StopwatchAction::allowed_values());
    };

    let mut args = Map::new();
    args.insert("Status".to_string(), Value::from(parsed.status()));

    dispatch_command(&state, PixooCommand::ToolsStopwatch, args).await
}

#[tracing::instrument(skip(state, payload))]
async fn scoreboard(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ScoreboardRequest>,
) -> Response {
    if let Err(errors) = payload.validate() {
        return validation_errors_response(&errors);
    }

    let mut args = Map::new();
    args.insert("BlueScore".to_string(), Value::from(payload.blue_score));
    args.insert("RedScore".to_string(), Value::from(payload.red_score));

    dispatch_command(&state, PixooCommand::ToolsScoreboard, args).await
}

#[tracing::instrument(skip(state))]
async fn soundmeter(State(state): State<Arc<AppState>>, Path(action): Path<String>) -> Response {
    let Ok(parsed) = action.parse::<SoundmeterAction>() else {
        return action_validation_error(&action, SoundmeterAction::allowed_values());
    };

    let mut args = Map::new();
    args.insert("NoiseStatus".to_string(), Value::from(parsed.status()));

    dispatch_command(&state, PixooCommand::ToolsSoundMeter, args).await
}

async fn dispatch_command(
    state: &AppState,
    command: PixooCommand,
    args: Map<String, Value>,
) -> Response {
    let client = &state.pixoo_client;
    match client.send_command(&command, args).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => {
            let (status, body) = map_pixoo_error(&err, &format!("Pixoo {command} command"));
            error!(command = %command, error = ?err, status = %status, "Pixoo tool command failed");
            (status, body).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mount_tool_routes;
    use crate::pixoo::{PixooClient, PixooClientConfig};
    use crate::routes::common::testing::send_json_request;
    use crate::state::AppState;
    use axum::http::{Method, StatusCode};
    use axum::Router;
    use httpmock::{Method as MockMethod, MockServer};
    use serde_json::{json, Value};
    use std::sync::Arc;

    fn build_tool_app(state: Arc<AppState>) -> Router {
        mount_tool_routes(Router::new()).with_state(state)
    }

    fn assert_validation_failed(body: &str) -> Value {
        let parsed: Value = serde_json::from_str(body).unwrap();
        assert_eq!(parsed["error"], "validation failed");
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
        assert_eq!(json_body["error_code"], 1);
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
