use crate::pixoo::client::PixooResponse;
use crate::pixoo::fields::{request as req, response as resp};
use crate::pixoo::PixooCommand;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use chrono::{NaiveDateTime, TimeZone, Utc};
use serde::Serialize;
use serde_json::{Map, Value};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error};

use crate::routes::common::{
    dispatch_pixoo_command, dispatch_pixoo_query, internal_server_error, service_unavailable,
    validation_error_simple,
};

use super::parsing::{parse_i64, parse_string};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManageTime {
    utc_time: String,
    local_time: String,
}

#[tracing::instrument(skip(state))]
pub async fn manage_time(State(state): State<Arc<AppState>>) -> Response {
    let response = match dispatch_pixoo_query(&state, PixooCommand::ManageGetTime).await {
        Ok(resp) => resp,
        Err(err) => return err,
    };

    match map_time(&response) {
        Ok(time) => axum::Json(time).into_response(),
        Err(msg) => {
            error!(error = %msg, "failed to map time response");
            service_unavailable()
        }
    }
}

#[tracing::instrument(skip(state))]
pub async fn manage_set_time(State(state): State<Arc<AppState>>) -> Response {
    let utc_secs = match current_utc_seconds() {
        Ok(secs) => secs,
        Err(err) => {
            error!(error = %err, "failed to read system UTC");
            return internal_server_error("failed to compute UTC timestamp");
        }
    };

    debug!(utc = utc_secs, "setting device UTC clock");
    let mut args = Map::new();
    args.insert(req::UTC.to_string(), Value::from(utc_secs));

    dispatch_pixoo_command(&state, PixooCommand::ManageSetUtc, args).await
}

#[tracing::instrument(skip(state))]
pub async fn manage_set_timezone(
    State(state): State<Arc<AppState>>,
    Path(offset): Path<String>,
) -> Response {
    let offset_value = match parse_timezone_offset(&offset) {
        Ok(value) => value,
        Err(message) => return offset_validation_error(&message),
    };
    // Pixoo expects the opposite sign from what humans call "timezone offset",
    // so sending +3 requires the command `GMT-3` when interacting with `/post`.
    let timezone_value = match offset_value {
        0 => "GMT+0".to_string(),
        value if value > 0 => format!("GMT-{value}"),
        value => format!("GMT+{}", value.abs()),
    };

    let mut args = Map::new();
    args.insert(
        req::TIMEZONE_VALUE.to_string(),
        Value::String(timezone_value),
    );

    dispatch_pixoo_command(&state, PixooCommand::ManageSetTimezone, args).await
}

#[tracing::instrument(skip(state))]
pub async fn manage_set_time_mode(
    State(state): State<Arc<AppState>>,
    Path(mode): Path<String>,
) -> Response {
    let mode_value = match mode.to_ascii_lowercase().as_str() {
        "12h" => 0,
        "24h" => 1,
        _ => return validation_error_simple("mode", "mode must be '12h' or '24h'"),
    };

    let mut args = Map::new();
    args.insert(req::MODE.to_string(), Value::from(mode_value));

    dispatch_pixoo_command(&state, PixooCommand::ManageSetTimeMode, args).await
}

pub fn time_mode(response: &PixooResponse) -> Result<String, String> {
    let flag = parse_string(response, resp::TIME_24_FLAG)?;
    Ok(if flag == "1" {
        "TWENTY_FOUR".to_string()
    } else {
        "TWELVE".to_string()
    })
}

fn offset_validation_error(message: &str) -> Response {
    validation_error_simple("offset", message)
}

fn parse_timezone_offset(value: &str) -> Result<i8, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("offset must be provided".to_string());
    }

    let parsed = trimmed
        .parse::<i32>()
        .map_err(|_| "offset must be an integer".to_string())?;

    if !(-12..=14).contains(&parsed) {
        return Err("offset must be between -12 and 14".to_string());
    }

    Ok(i8::try_from(parsed).unwrap())
}

fn current_utc_seconds() -> Result<i64, String> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| err.to_string())?;
    let secs = i64::try_from(duration.as_secs()).map_err(|err| err.to_string())?;
    Ok(secs)
}

fn map_time(response: &PixooResponse) -> Result<ManageTime, String> {
    let utc_secs = parse_i64(response, resp::UTC_TIME)?;
    let utc_time = Utc
        .timestamp_opt(utc_secs, 0)
        .single()
        .ok_or_else(|| format!("UTCTime {utc_secs} out of range"))?;
    let utc_iso = utc_time.format("%Y-%m-%dT%H:%M:%S").to_string();

    let local_value = parse_string(response, resp::LOCAL_TIME)?;
    let local_naive = NaiveDateTime::parse_from_str(&local_value, "%Y-%m-%d %H:%M:%S")
        .map_err(|err| format!("LocalTime parse error: {err}"))?;
    let local_iso = local_naive.format("%Y-%m-%dT%H:%M:%S").to_string();

    Ok(ManageTime {
        utc_time: utc_iso,
        local_time: local_iso,
    })
}

#[cfg(test)]
mod tests {
    use crate::pixoo::{PixooClient, PixooClientConfig};
    use crate::routes::common::testing::send_json_request;
    use crate::routes::manage::mount_manage_routes;
    use crate::state::AppState;
    use axum::http::{Method, StatusCode};
    use axum::Router;
    use chrono::{TimeZone, Utc};
    use httpmock::{Method as MockMethod, MockServer};
    use serde_json::{json, Value};
    use std::sync::Arc;

    fn build_manage_app(state: Arc<AppState>) -> Router {
        mount_manage_routes(Router::new()).with_state(state)
    }

    async fn send_get(app: &Router, uri: &str) -> (StatusCode, String) {
        send_json_request(app, Method::GET, uri, None).await
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
    async fn time_returns_iso_strings() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(MockMethod::POST).path("/post");
            then.status(200).body(
                json!({
                    "error_code": 0,
                    "UTCTime": 1_700_000_000,
                    "LocalTime": "2023-05-05 13:30:00",
                })
                .to_string(),
            );
        });

        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, body) = send_get(&app, "/manage/time").await;

        assert_eq!(status, StatusCode::OK);
        let json_body: Value = serde_json::from_str(&body).unwrap();
        let utc_expected = Utc
            .timestamp_opt(1_700_000_000, 0)
            .single()
            .expect("valid utc time")
            .format("%Y-%m-%dT%H:%M:%S")
            .to_string();
        assert_eq!(json_body["utcTime"], utc_expected);
        assert_eq!(json_body["localTime"], "2023-05-05T13:30:00");
    }

    #[tokio::test]
    async fn time_handles_failure() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(MockMethod::POST).path("/post");
            then.status(200).body(r#"{"error_code":1}"#);
        });

        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, body) = send_get(&app, "/manage/time").await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        let json_body: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json_body["error_kind"], "device-error");
        assert_eq!(json_body["error_status"], 503);
        assert_eq!(json_body["error_code"], 1);
    }

    #[tokio::test]
    async fn timezone_sets_offset() {
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(MockMethod::POST)
                .path("/post")
                .body_includes("\"Command\":\"Sys/TimeZone\"")
                .body_includes("\"TimeZoneValue\":\"GMT+5\"");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, _) = send_post(&app, "/manage/time/offset/-5", None).await;

        assert_eq!(status, StatusCode::OK);
        mock.assert();
    }

    #[tokio::test]
    async fn timezone_positive_offset_inverts_sign() {
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(MockMethod::POST)
                .path("/post")
                .body_includes("\"Command\":\"Sys/TimeZone\"")
                .body_includes("\"TimeZoneValue\":\"GMT-3\"");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, _) = send_post(&app, "/manage/time/offset/+3", None).await;

        assert_eq!(status, StatusCode::OK);
        mock.assert();
    }

    #[tokio::test]
    async fn timezone_invalid_offset_out_of_range() {
        let server = MockServer::start_async().await;
        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, body) = send_post(&app, "/manage/time/offset/20", None).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        let json_body: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json_body["error"], "validation failed");
        assert_eq!(
            json_body["details"]["offset"],
            "offset must be between -12 and 14"
        );
    }

    #[tokio::test]
    async fn timezone_invalid_offset_non_numeric() {
        let server = MockServer::start_async().await;
        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, body) = send_post(&app, "/manage/time/offset/abc", None).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        let json_body: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json_body["error"], "validation failed");
        assert_eq!(json_body["details"]["offset"], "offset must be an integer");
    }

    #[tokio::test]
    async fn manage_time_sets_current_utc() {
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(MockMethod::POST)
                .path("/post")
                .body_includes("\"Command\":\"Device/SetUTC\"")
                .body_includes("\"Utc\":");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, _) = send_post(&app, "/manage/time", None).await;

        assert_eq!(status, StatusCode::OK);
        mock.assert();
    }

    #[tokio::test]
    async fn time_mode_sets_12h() {
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(MockMethod::POST)
                .path("/post")
                .body_includes("\"Command\":\"Device/SetTime24Flag\"")
                .body_includes("\"Mode\":0");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, _) =
            send_json_request(&app, Method::POST, "/manage/time/mode/12h", None).await;

        assert_eq!(status, StatusCode::OK);
        mock.assert();
    }

    #[tokio::test]
    async fn time_mode_sets_24h() {
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(MockMethod::POST)
                .path("/post")
                .body_includes("\"Command\":\"Device/SetTime24Flag\"")
                .body_includes("\"Mode\":1");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, _) =
            send_json_request(&app, Method::POST, "/manage/time/mode/24h", None).await;

        assert_eq!(status, StatusCode::OK);
        mock.assert();
    }

    #[tokio::test]
    async fn time_mode_rejects_invalid() {
        let server = MockServer::start_async().await;
        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, body) =
            send_json_request(&app, Method::POST, "/manage/time/mode/invalid", None).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        let json_body: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json_body["error"], "validation failed");
        assert_eq!(json_body["details"]["mode"], "mode must be '12h' or '24h'");
    }
}
