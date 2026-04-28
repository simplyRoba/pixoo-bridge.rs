use crate::pixoo::fields::request as req;
use crate::pixoo::PixooCommand;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::response::Response;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::str::FromStr;
use std::sync::Arc;
use validator::Validate;

use crate::routes::common::{
    dispatch_pixoo_command, validation_error_simple, PathParam, ValidatedJson, ValidatedPath,
};

#[derive(Debug, Deserialize, Validate)]
pub struct WhiteBalanceRequest {
    #[validate(range(min = 0, max = 100))]
    pub red: i64,
    #[validate(range(min = 0, max = 100))]
    pub green: i64,
    #[validate(range(min = 0, max = 100))]
    pub blue: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OnOffAction {
    On,
    Off,
}

impl OnOffAction {
    pub fn flag_value(&self) -> i32 {
        match self {
            Self::On => 1,
            Self::Off => 0,
        }
    }
}

impl PathParam for OnOffAction {
    fn allowed_values() -> &'static [&'static str] {
        &["on", "off"]
    }
}

impl FromStr for OnOffAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "on" => Ok(Self::On),
            "off" => Ok(Self::Off),
            _ => Err(()),
        }
    }
}

#[tracing::instrument(skip(state))]
pub async fn manage_display_on(
    State(state): State<Arc<AppState>>,
    ValidatedPath(action): ValidatedPath<OnOffAction>,
) -> Response {
    let mut args = Map::new();
    args.insert(req::ON_OFF.to_string(), Value::from(action.flag_value()));

    dispatch_pixoo_command(&state, PixooCommand::ManageDisplayPower, args).await
}

#[tracing::instrument(skip(state))]
pub async fn manage_display_brightness(
    State(state): State<Arc<AppState>>,
    Path(value): Path<String>,
) -> Response {
    let brightness_value = match value.parse::<i32>() {
        Ok(val) if (0..=100).contains(&val) => val,
        _ => return validation_error_simple("value", "value must be an integer between 0 and 100"),
    };

    let mut args = Map::new();
    args.insert(req::BRIGHTNESS.to_string(), Value::from(brightness_value));

    dispatch_pixoo_command(&state, PixooCommand::ManageDisplayBrightness, args).await
}

#[tracing::instrument(skip(state))]
pub async fn manage_display_rotation(
    State(state): State<Arc<AppState>>,
    Path(angle): Path<String>,
) -> Response {
    let mode_value = match angle.parse::<i32>() {
        Ok(val) if [0, 90, 180, 270].contains(&val) => val / 90,
        _ => return validation_error_simple("angle", "angle must be 0, 90, 180, or 270"),
    };

    let mut args = Map::new();
    args.insert(req::MODE.to_string(), Value::from(mode_value));

    dispatch_pixoo_command(&state, PixooCommand::ManageDisplayRotation, args).await
}

#[tracing::instrument(skip(state))]
pub async fn manage_display_mirror(
    State(state): State<Arc<AppState>>,
    ValidatedPath(action): ValidatedPath<OnOffAction>,
) -> Response {
    let mut args = Map::new();
    args.insert(req::MODE.to_string(), Value::from(action.flag_value()));

    dispatch_pixoo_command(&state, PixooCommand::ManageDisplayMirror, args).await
}

#[tracing::instrument(skip(state))]
pub async fn manage_display_overclock(
    State(state): State<Arc<AppState>>,
    ValidatedPath(action): ValidatedPath<OnOffAction>,
) -> Response {
    let mut args = Map::new();
    args.insert(req::MODE.to_string(), Value::from(action.flag_value()));

    dispatch_pixoo_command(&state, PixooCommand::ManageDisplayOverclock, args).await
}

#[tracing::instrument(skip(state, payload))]
pub async fn manage_display_white_balance(
    State(state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<WhiteBalanceRequest>,
) -> Response {
    let mut args = Map::new();
    args.insert(req::R_VALUE.to_string(), Value::from(payload.red));
    args.insert(req::G_VALUE.to_string(), Value::from(payload.green));
    args.insert(req::B_VALUE.to_string(), Value::from(payload.blue));

    dispatch_pixoo_command(&state, PixooCommand::ManageDisplayWhiteBalance, args).await
}

#[cfg(test)]
mod tests {
    use crate::pixoo::{PixooClient, PixooClientConfig};
    use crate::routes::common::testing::send_json_request;
    use crate::routes::manage::mount_manage_routes;
    use crate::state::AppState;
    use axum::http::{Method, StatusCode};
    use axum::Router;
    use httpmock::{Method as MockMethod, MockServer};
    use serde_json::{json, Value};
    use std::sync::Arc;

    fn build_manage_app(state: Arc<AppState>) -> Router {
        mount_manage_routes(Router::new()).with_state(state)
    }

    async fn send_post(
        app: &Router,
        uri: &str,
        body: Option<serde_json::Value>,
    ) -> (StatusCode, String) {
        send_json_request(app, Method::POST, uri, body).await
    }

    fn manage_state_with_client(base_url: &str) -> Arc<AppState> {
        let client = PixooClient::new(base_url, PixooClientConfig::default()).expect("client");
        Arc::new(AppState::with_client(client))
    }

    #[tokio::test]
    async fn display_on_toggles_power() {
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(MockMethod::POST)
                .path("/post")
                .body_includes("\"Command\":\"Channel/OnOffScreen\"")
                .body_includes("\"OnOff\":1");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, _) = send_json_request(&app, Method::POST, "/manage/display/on", None).await;

        assert_eq!(status, StatusCode::OK);
        mock.assert();
    }

    #[tokio::test]
    async fn display_on_rejects_invalid_action() {
        let server = MockServer::start_async().await;
        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, body) =
            send_json_request(&app, Method::POST, "/manage/display/invalid", None).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        let json_body: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json_body["error"], "validation failed");
        assert_eq!(json_body["details"]["action"]["provided"], "invalid");
        assert_eq!(
            json_body["details"]["action"]["allowed"],
            json!(["on", "off"])
        );
    }

    #[tokio::test]
    async fn display_brightness_sets_value() {
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(MockMethod::POST)
                .path("/post")
                .body_includes("\"Command\":\"Channel/SetBrightness\"")
                .body_includes("\"Brightness\":75");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, _) =
            send_json_request(&app, Method::POST, "/manage/display/brightness/75", None).await;

        assert_eq!(status, StatusCode::OK);
        mock.assert();
    }

    #[tokio::test]
    async fn display_brightness_rejects_out_of_range() {
        let server = MockServer::start_async().await;
        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, body) =
            send_json_request(&app, Method::POST, "/manage/display/brightness/150", None).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        let json_body: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json_body["error"], "validation failed");
        assert_eq!(
            json_body["details"]["value"],
            "value must be an integer between 0 and 100"
        );
    }

    #[tokio::test]
    async fn display_rotation_sets_angle() {
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(MockMethod::POST)
                .path("/post")
                .body_includes("\"Command\":\"Device/SetScreenRotationAngle\"")
                .body_includes("\"Mode\":1");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, _) =
            send_json_request(&app, Method::POST, "/manage/display/rotation/90", None).await;

        assert_eq!(status, StatusCode::OK);
        mock.assert();
    }

    #[tokio::test]
    async fn display_rotation_rejects_invalid_angle() {
        let server = MockServer::start_async().await;
        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, body) =
            send_json_request(&app, Method::POST, "/manage/display/rotation/45", None).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        let json_body: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json_body["error"], "validation failed");
        assert_eq!(
            json_body["details"]["angle"],
            "angle must be 0, 90, 180, or 270"
        );
    }

    #[tokio::test]
    async fn display_mirror_toggles_mode() {
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(MockMethod::POST)
                .path("/post")
                .body_includes("\"Command\":\"Device/SetMirrorMode\"")
                .body_includes("\"Mode\":1");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, _) =
            send_json_request(&app, Method::POST, "/manage/display/mirror/on", None).await;

        assert_eq!(status, StatusCode::OK);
        mock.assert();
    }

    #[tokio::test]
    async fn display_mirror_rejects_invalid_action() {
        let server = MockServer::start_async().await;
        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, body) =
            send_json_request(&app, Method::POST, "/manage/display/mirror/invalid", None).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        let json_body: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json_body["error"], "validation failed");
        assert_eq!(json_body["details"]["action"]["provided"], "invalid");
        assert_eq!(
            json_body["details"]["action"]["allowed"],
            json!(["on", "off"])
        );
    }

    #[tokio::test]
    async fn display_overclock_toggles_mode() {
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(MockMethod::POST)
                .path("/post")
                .body_includes("\"Command\":\"Device/SetHighLightMode\"")
                .body_includes("\"Mode\":1");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, _) = send_json_request(
            &app,
            Method::POST,
            "/manage/display/brightness/overclock/on",
            None,
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        mock.assert();
    }

    #[tokio::test]
    async fn display_overclock_rejects_invalid_action() {
        let server = MockServer::start_async().await;
        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, body) = send_json_request(
            &app,
            Method::POST,
            "/manage/display/brightness/overclock/invalid",
            None,
        )
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        let json_body: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json_body["error"], "validation failed");
        assert_eq!(json_body["details"]["action"]["provided"], "invalid");
        assert_eq!(
            json_body["details"]["action"]["allowed"],
            json!(["on", "off"])
        );
    }

    #[tokio::test]
    async fn display_white_balance_sets_values() {
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(MockMethod::POST)
                .path("/post")
                .body_includes("\"Command\":\"Device/SetWhiteBalance\"")
                .body_includes("\"RValue\":90")
                .body_includes("\"GValue\":100")
                .body_includes("\"BValue\":100");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, _) = send_post(
            &app,
            "/manage/display/white-balance",
            Some(json!({ "red": 90, "green": 100, "blue": 100 })),
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        mock.assert();
    }

    #[tokio::test]
    async fn display_white_balance_rejects_out_of_range() {
        let server = MockServer::start_async().await;
        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, body) = send_post(
            &app,
            "/manage/display/white-balance",
            Some(json!({ "red": 150, "green": 100, "blue": 100 })),
        )
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        let json_body: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json_body["error"], "validation failed");
        assert!(json_body["details"]["red"]
            .as_array()
            .unwrap()
            .contains(&Value::String("range".to_string())));
    }

    #[tokio::test]
    async fn display_off_toggles_power() {
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(MockMethod::POST)
                .path("/post")
                .body_includes("\"Command\":\"Channel/OnOffScreen\"")
                .body_includes("\"OnOff\":0");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, _) = send_json_request(&app, Method::POST, "/manage/display/off", None).await;

        assert_eq!(status, StatusCode::OK);
        mock.assert();
    }

    #[tokio::test]
    async fn display_brightness_rejects_non_numeric() {
        let server = MockServer::start_async().await;
        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, body) =
            send_json_request(&app, Method::POST, "/manage/display/brightness/abc", None).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        let json_body: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json_body["error"], "validation failed");
        assert_eq!(
            json_body["details"]["value"],
            "value must be an integer between 0 and 100",
        );
    }

    #[tokio::test]
    async fn display_mirror_sets_mode_off() {
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(MockMethod::POST)
                .path("/post")
                .body_includes("\"Command\":\"Device/SetMirrorMode\"")
                .body_includes("\"Mode\":0");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, _) =
            send_json_request(&app, Method::POST, "/manage/display/mirror/off", None).await;

        assert_eq!(status, StatusCode::OK);
        mock.assert();
    }

    #[tokio::test]
    async fn display_overclock_sets_mode_off() {
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(MockMethod::POST)
                .path("/post")
                .body_includes("\"Command\":\"Device/SetHighLightMode\"")
                .body_includes("\"Mode\":0");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, _) = send_json_request(
            &app,
            Method::POST,
            "/manage/display/brightness/overclock/off",
            None,
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        mock.assert();
    }

    #[tokio::test]
    async fn display_white_balance_rejects_missing_values() {
        let server = MockServer::start_async().await;
        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, body) = send_post(
            &app,
            "/manage/display/white-balance",
            Some(json!({ "red": 100 })),
        )
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(body.contains("missing"));
    }
}
