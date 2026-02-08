use crate::pixoo::client::PixooResponse;
use crate::pixoo::{map_pixoo_error, PixooCommand};
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use chrono::{NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error};
use validator::{Validate, ValidationError, ValidationErrors};

use crate::state::AppState;

pub fn mount_manage_routes(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router
        .route("/manage/settings", get(manage_settings))
        .route("/manage/time", get(manage_time).post(manage_set_time))
        .route("/manage/weather", get(manage_weather))
        .route("/manage/weather/location", post(manage_set_location))
        .route("/manage/time/offset/{offset}", post(manage_set_timezone))
        .route("/manage/time/mode/{mode}", post(manage_set_time_mode))
        .route(
            "/manage/weather/temperature-unit/{unit}",
            post(manage_set_temperature_unit),
        )
        .route("/manage/display/{action}", post(manage_display_on))
        .route(
            "/manage/display/brightness/{value}",
            post(manage_display_brightness),
        )
        .route(
            "/manage/display/rotation/{angle}",
            post(manage_display_rotation),
        )
        .route(
            "/manage/display/mirror/{action}",
            post(manage_display_mirror),
        )
        .route(
            "/manage/display/brightness/overclock/{action}",
            post(manage_display_overclock),
        )
        .route(
            "/manage/display/white-balance",
            post(manage_display_white_balance),
        )
}

#[tracing::instrument(skip(state))]
async fn manage_settings(State(state): State<Arc<AppState>>) -> Response {
    let response = match dispatch_manage_command(&state, PixooCommand::ManageGetSettings).await {
        Ok(response) => response,
        Err(resp) => return resp,
    };

    match map_settings(&response) {
        Ok(payload) => Json(payload).into_response(),
        Err(err) => {
            error!(error = %err, "failed to map settings response");
            service_unavailable()
        }
    }
}

#[tracing::instrument(skip(state))]
async fn manage_time(State(state): State<Arc<AppState>>) -> Response {
    let response = match dispatch_manage_command(&state, PixooCommand::ManageGetTime).await {
        Ok(response) => response,
        Err(resp) => return resp,
    };

    match map_time(&response) {
        Ok(payload) => Json(payload).into_response(),
        Err(err) => {
            error!(error = %err, "failed to map time response");
            service_unavailable()
        }
    }
}

#[tracing::instrument(skip(state))]
async fn manage_weather(State(state): State<Arc<AppState>>) -> Response {
    let response = match dispatch_manage_command(&state, PixooCommand::ManageGetWeather).await {
        Ok(response) => response,
        Err(resp) => return resp,
    };

    match map_weather(&response) {
        Ok(payload) => Json(payload).into_response(),
        Err(err) => {
            error!(error = %err, "failed to map weather response");
            service_unavailable()
        }
    }
}

#[tracing::instrument(skip(state, payload))]
async fn manage_set_location(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LocationRequest>,
) -> Response {
    if let Err(errors) = payload.validate() {
        return validation_errors_response(&errors);
    }

    let mut args = Map::new();
    args.insert(
        "Longitude".to_string(),
        Value::String(payload.longitude.to_string()),
    );
    args.insert(
        "Latitude".to_string(),
        Value::String(payload.latitude.to_string()),
    );

    dispatch_manage_post_command(&state, PixooCommand::ManageSetLocation, args).await
}

#[tracing::instrument(skip(state))]
async fn manage_set_time(State(state): State<Arc<AppState>>) -> Response {
    let utc_secs = match current_utc_seconds() {
        Ok(secs) => secs,
        Err(err) => {
            error!(error = %err, "failed to read system UTC");
            return internal_server_error("failed to compute UTC timestamp");
        }
    };

    debug!(utc = utc_secs, "setting device UTC clock");
    let mut args = Map::new();
    args.insert("Utc".to_string(), Value::from(utc_secs));

    dispatch_manage_post_command(&state, PixooCommand::ManageSetUtc, args).await
}

#[tracing::instrument(skip(state))]
async fn manage_set_timezone(
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
    args.insert("TimeZoneValue".to_string(), Value::String(timezone_value));

    dispatch_manage_post_command(&state, PixooCommand::ManageSetTimezone, args).await
}

#[tracing::instrument(skip(state))]
async fn manage_set_time_mode(
    State(state): State<Arc<AppState>>,
    Path(mode): Path<String>,
) -> Response {
    let mode_value = match mode.as_str() {
        "12h" => 0,
        "24h" => 1,
        _ => return validation_error_simple("mode", "mode must be '12h' or '24h'"),
    };

    let mut args = Map::new();
    args.insert("Mode".to_string(), Value::from(mode_value));

    dispatch_manage_post_command(&state, PixooCommand::ManageSetTimeMode, args).await
}

#[tracing::instrument(skip(state))]
async fn manage_set_temperature_unit(
    State(state): State<Arc<AppState>>,
    Path(unit): Path<String>,
) -> Response {
    let mode_value = match unit.as_str() {
        "celsius" => 0,
        "fahrenheit" => 1,
        _ => return validation_error_simple("unit", "unit must be 'celsius' or 'fahrenheit'"),
    };

    let mut args = Map::new();
    args.insert("Mode".to_string(), Value::from(mode_value));

    dispatch_manage_post_command(&state, PixooCommand::ManageSetTemperatureUnit, args).await
}

#[tracing::instrument(skip(state))]
async fn manage_display_on(
    State(state): State<Arc<AppState>>,
    Path(action): Path<String>,
) -> Response {
    let Ok(parsed) = action.parse::<OnOffAction>() else {
        return action_validation_error(&action, OnOffAction::allowed_values());
    };

    let mut args = Map::new();
    args.insert("OnOff".to_string(), Value::from(parsed.flag_value()));

    dispatch_manage_post_command(&state, PixooCommand::ManageDisplayPower, args).await
}

#[tracing::instrument(skip(state))]
async fn manage_display_brightness(
    State(state): State<Arc<AppState>>,
    Path(value): Path<String>,
) -> Response {
    let brightness_value = match value.parse::<i32>() {
        Ok(val) if (0..=100).contains(&val) => val,
        _ => return validation_error_simple("value", "value must be an integer between 0 and 100"),
    };

    let mut args = Map::new();
    args.insert("Brightness".to_string(), Value::from(brightness_value));

    dispatch_manage_post_command(&state, PixooCommand::ManageDisplayBrightness, args).await
}

#[tracing::instrument(skip(state))]
async fn manage_display_rotation(
    State(state): State<Arc<AppState>>,
    Path(angle): Path<String>,
) -> Response {
    let mode_value = match angle.parse::<i32>() {
        Ok(val) if [0, 90, 180, 270].contains(&val) => val / 90,
        _ => return validation_error_simple("angle", "angle must be 0, 90, 180, or 270"),
    };

    let mut args = Map::new();
    args.insert("Mode".to_string(), Value::from(mode_value));

    dispatch_manage_post_command(&state, PixooCommand::ManageDisplayRotation, args).await
}

#[tracing::instrument(skip(state))]
async fn manage_display_mirror(
    State(state): State<Arc<AppState>>,
    Path(action): Path<String>,
) -> Response {
    let Ok(parsed) = action.parse::<OnOffAction>() else {
        return action_validation_error(&action, OnOffAction::allowed_values());
    };

    let mut args = Map::new();
    args.insert("Mode".to_string(), Value::from(parsed.flag_value()));

    dispatch_manage_post_command(&state, PixooCommand::ManageDisplayMirror, args).await
}

#[tracing::instrument(skip(state))]
async fn manage_display_overclock(
    State(state): State<Arc<AppState>>,
    Path(action): Path<String>,
) -> Response {
    let Ok(parsed) = action.parse::<OnOffAction>() else {
        return action_validation_error(&action, OnOffAction::allowed_values());
    };

    let mut args = Map::new();
    args.insert("Mode".to_string(), Value::from(parsed.flag_value()));

    dispatch_manage_post_command(&state, PixooCommand::ManageDisplayOverclock, args).await
}

#[derive(Debug, Deserialize, Validate)]
struct WhiteBalanceRequest {
    #[validate(range(min = 0, max = 100))]
    red: i32,
    #[validate(range(min = 0, max = 100))]
    green: i32,
    #[validate(range(min = 0, max = 100))]
    blue: i32,
}

#[tracing::instrument(skip(state, payload))]
async fn manage_display_white_balance(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Value>,
) -> Response {
    let payload = match serde_json::from_value::<WhiteBalanceRequest>(payload) {
        Ok(request) => request,
        Err(err) => {
            let message = err.to_string();
            return validation_error_simple("body", &message);
        }
    };

    if let Err(errors) = payload.validate() {
        return validation_errors_response(&errors);
    }

    let mut args = Map::new();
    args.insert("RValue".to_string(), Value::from(payload.red));
    args.insert("GValue".to_string(), Value::from(payload.green));
    args.insert("BValue".to_string(), Value::from(payload.blue));

    dispatch_manage_post_command(&state, PixooCommand::ManageDisplayWhiteBalance, args).await
}

async fn dispatch_manage_command(
    state: &AppState,
    command: PixooCommand,
) -> Result<PixooResponse, Response> {
    let client = &state.pixoo_client;
    match client.send_command(command.clone(), Map::new()).await {
        Ok(response) => Ok(response),
        Err(err) => {
            let (status, body) = map_pixoo_error(&err, &format!("Pixoo {command} command"));
            error!(command = %command, error = ?err, status = %status, "Pixoo manage command failed");
            Err((status, body).into_response())
        }
    }
}

fn service_unavailable() -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({ "error": "Pixoo command failed" })),
    )
        .into_response()
}

fn internal_server_error(message: &str) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": message })),
    )
        .into_response()
}

fn validation_error_message(error: &ValidationError) -> String {
    if let Some(message) = &error.message {
        message.to_string()
    } else {
        error.code.to_string()
    }
}

fn validation_error_response(details: Map<String, Value>) -> Response {
    let body = json!({
        "error": "validation failed",
        "details": Value::Object(details),
    });

    (StatusCode::BAD_REQUEST, Json(body)).into_response()
}

fn validation_errors_response(errors: &ValidationErrors) -> Response {
    let mut details = Map::new();

    for (field, field_errors) in errors.field_errors() {
        let messages: Vec<Value> = field_errors
            .iter()
            .map(|error| Value::String(validation_error_message(error)))
            .collect();
        details.insert(field.to_string(), Value::Array(messages));
    }

    validation_error_response(details)
}

fn offset_validation_error(message: &str) -> Response {
    validation_error_simple("offset", message)
}

fn validation_error_simple(field: &str, message: &str) -> Response {
    let mut details = Map::new();
    details.insert(field.to_string(), Value::String(message.to_string()));
    validation_error_response(details)
}

fn action_validation_error(action: &str, allowed: &[&str]) -> Response {
    let mut details = Map::new();
    details.insert(
        "action".to_string(),
        json!({
            "provided": action,
            "allowed": allowed,
        }),
    );

    validation_error_response(details)
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

    #[allow(clippy::cast_possible_truncation)]
    Ok(parsed as i8)
}

fn current_utc_seconds() -> Result<i64, String> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| err.to_string())?;
    let secs = i64::try_from(duration.as_secs()).map_err(|err| err.to_string())?;
    Ok(secs)
}

async fn dispatch_manage_post_command(
    state: &AppState,
    command: PixooCommand,
    args: Map<String, Value>,
) -> Response {
    let client = &state.pixoo_client;
    match client.send_command(command.clone(), args).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => {
            let (status, body) = map_pixoo_error(&err, &format!("Pixoo {command} command"));
            error!(command = %command, error = ?err, status = %status, "Pixoo manage command failed");
            (status, body).into_response()
        }
    }
}

fn map_settings(response: &PixooResponse) -> Result<ManageSettings, String> {
    Ok(ManageSettings {
        display_on: flag_bool(response, "LightSwitch")?,
        brightness: parse_i64(response, "Brightness")?,
        time_mode: time_mode(response)?,
        rotation_angle: rotation_angle(response)?,
        mirrored: flag_bool(response, "MirrorFlag")?,
        temperature_unit: temperature_unit(response)?,
        current_clock_id: parse_i64(response, "CurClockId")?,
    })
}

fn map_time(response: &PixooResponse) -> Result<ManageTime, String> {
    let utc_secs = parse_i64(response, "UTCTime")?;
    let utc_time = Utc
        .timestamp_opt(utc_secs, 0)
        .single()
        .ok_or_else(|| format!("UTCTime {utc_secs} out of range"))?;
    let utc_iso = utc_time.format("%Y-%m-%dT%H:%M:%S").to_string();

    let local_value = parse_string(response, "LocalTime")?;
    let local_naive = NaiveDateTime::parse_from_str(&local_value, "%Y-%m-%d %H:%M:%S")
        .map_err(|err| format!("LocalTime parse error: {err}"))?;
    let local_iso = local_naive.format("%Y-%m-%dT%H:%M:%S").to_string();

    Ok(ManageTime {
        utc_time: utc_iso,
        local_time: local_iso,
    })
}

fn map_weather(response: &PixooResponse) -> Result<ManageWeather, String> {
    Ok(ManageWeather {
        weather_string: parse_string(response, "Weather")?,
        current_temperature: parse_f64(response, "CurTemp")?,
        minimal_temperature: parse_f64(response, "MinTemp")?,
        maximal_temperature: parse_f64(response, "MaxTemp")?,
        pressure: parse_i64(response, "Pressure")?,
        humidity: parse_i64(response, "Humidity")?,
        wind_speed: parse_f64(response, "WindSpeed")?,
    })
}

fn parse_string(response: &PixooResponse, key: &str) -> Result<String, String> {
    response
        .get(key)
        .ok_or_else(|| format!("missing field {key}"))
        .and_then(|value| match value {
            Value::String(text) => Ok(text.clone()),
            Value::Number(number) => Ok(number.to_string()),
            other => Err(format!("unexpected type for {key}: {other}")),
        })
}

fn flag_bool(response: &PixooResponse, key: &str) -> Result<bool, String> {
    Ok(parse_string(response, key)? == "1")
}

fn parse_i64(response: &PixooResponse, key: &str) -> Result<i64, String> {
    let value = parse_string(response, key)?;
    value
        .parse::<i64>()
        .map_err(|err| format!("{key} is not an integer: {err}"))
}

fn parse_f64(response: &PixooResponse, key: &str) -> Result<f64, String> {
    let value = parse_string(response, key)?;
    value
        .parse::<f64>()
        .map_err(|err| format!("{key} is not a float: {err}"))
}

fn time_mode(response: &PixooResponse) -> Result<String, String> {
    let flag = parse_string(response, "Time24Flag")?;
    Ok(if flag == "1" {
        "TWENTY_FOUR".to_string()
    } else {
        "TWELVE".to_string()
    })
}

fn rotation_angle(response: &PixooResponse) -> Result<i64, String> {
    let flag = parse_string(response, "RotationFlag")?;
    if flag == "0" {
        return Ok(0);
    }
    let rotation = flag
        .parse::<i64>()
        .map_err(|err| format!("RotationFlag is not integer: {err}"))?;
    Ok(rotation * 90)
}

fn temperature_unit(response: &PixooResponse) -> Result<String, String> {
    let flag = parse_string(response, "TemperatureMode")?;
    Ok(if flag == "1" {
        "FAHRENHEIT".to_string()
    } else {
        "CELSIUS".to_string()
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ManageSettings {
    display_on: bool,
    brightness: i64,
    time_mode: String,
    rotation_angle: i64,
    mirrored: bool,
    temperature_unit: String,
    current_clock_id: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ManageTime {
    utc_time: String,
    local_time: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ManageWeather {
    weather_string: String,
    current_temperature: f64,
    minimal_temperature: f64,
    maximal_temperature: f64,
    pressure: i64,
    humidity: i64,
    wind_speed: f64,
}

#[derive(Debug, Deserialize, Validate)]
struct LocationRequest {
    #[validate(range(min = -180.0, max = 180.0))]
    longitude: f64,
    #[validate(range(min = -90.0, max = 90.0))]
    latitude: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum OnOffAction {
    On,
    Off,
}

impl OnOffAction {
    fn flag_value(&self) -> i32 {
        match self {
            Self::On => 1,
            Self::Off => 0,
        }
    }

    fn allowed_values() -> &'static [&'static str] {
        &["on", "off"]
    }
}

impl FromStr for OnOffAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "on" => Ok(Self::On),
            "off" => Ok(Self::Off),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mount_manage_routes;
    use crate::pixoo::{PixooClient, PixooClientConfig};
    use crate::state::AppState;
    use axum::body::{to_bytes, Body};
    use axum::http::{Method, Request, StatusCode};
    use axum::Router;
    use chrono::{TimeZone, Utc};
    use httpmock::{Method as MockMethod, MockServer};
    use serde_json::{json, Value};
    use std::sync::Arc;
    use tower::ServiceExt;

    fn build_manage_app(state: Arc<AppState>) -> Router {
        mount_manage_routes(Router::new()).with_state(state)
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
        Arc::new(AppState {
            health_forward: false,
            pixoo_client: client,
        })
    }

    #[tokio::test]
    async fn settings_returns_normalized_payload() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(MockMethod::POST).path("/post");
            then.status(200).body(
                json!({
                    "error_code": 0,
                    "LightSwitch": "1",
                    "Brightness": 80,
                    "Time24Flag": "1",
                    "RotationFlag": "3",
                    "MirrorFlag": "0",
                    "TemperatureMode": "1",
                    "CurClockId": 5,
                })
                .to_string(),
            );
        });

        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, body) = send_get(&app, "/manage/settings").await;

        assert_eq!(status, StatusCode::OK);
        let json_body: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json_body["displayOn"], true);
        assert_eq!(json_body["brightness"], 80);
        assert_eq!(json_body["timeMode"], "TWENTY_FOUR");
        assert_eq!(json_body["rotationAngle"], 270);
        assert_eq!(json_body["mirrored"], false);
        assert_eq!(json_body["temperatureUnit"], "FAHRENHEIT");
        assert_eq!(json_body["currentClockId"], 5);
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
    async fn weather_returns_normalized_fields() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(MockMethod::POST).path("/post");
            then.status(200).body(
                json!({
                    "error_code": 0,
                    "Weather": "Cloudy",
                    "CurTemp": 26.5,
                    "MinTemp": 24.0,
                    "MaxTemp": 28.1,
                    "Pressure": 1006,
                    "Humidity": 50,
                    "Visibility": 10000,
                    "WindSpeed": 2.54,
                })
                .to_string(),
            );
        });

        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, body) = send_get(&app, "/manage/weather").await;

        assert_eq!(status, StatusCode::OK);
        let json_body: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json_body["weatherString"], "Cloudy");
        assert_eq!(json_body["currentTemperature"], 26.5);
        assert_eq!(json_body["minimalTemperature"], 24.0);
        assert_eq!(json_body["maximalTemperature"], 28.1);
        assert_eq!(json_body["pressure"], 1006);
        assert_eq!(json_body["humidity"], 50);
        assert_eq!(json_body["windSpeed"], 2.54);
        assert!(json_body.get("visibility").is_none());
    }

    #[tokio::test]
    async fn weather_handles_failure() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(MockMethod::POST).path("/post");
            then.status(200).body(r#"{"error_code":1}"#);
        });

        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, body) = send_get(&app, "/manage/weather").await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        let json_body: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json_body["error_kind"], "device-error");
        assert_eq!(json_body["error_status"], 503);
        assert_eq!(json_body["error_code"], 1);
    }

    #[tokio::test]
    async fn location_sets_coordinates() {
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(MockMethod::POST)
                .path("/post")
                .body_includes("\"Command\":\"Sys/LogAndLat\"")
                .body_includes("\"Longitude\":\"30.29\"")
                .body_includes("\"Latitude\":\"20.58\"");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, body) = send_post(
            &app,
            "/manage/weather/location",
            Some(json!({ "longitude": 30.29, "latitude": 20.58 })),
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        assert!(body.is_empty());

        mock.assert();
    }

    #[tokio::test]
    async fn location_rejects_invalid_coordinate() {
        let server = MockServer::start_async().await;
        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, body) = send_post(
            &app,
            "/manage/weather/location",
            Some(json!({ "longitude": 190.0, "latitude": 20.58 })),
        )
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        let json_body: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json_body["error"], "validation failed");
        assert!(json_body["details"]["longitude"]
            .as_array()
            .unwrap()
            .contains(&Value::String("range".to_string())));
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

    #[tokio::test]
    async fn temp_unit_sets_celsius() {
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(MockMethod::POST)
                .path("/post")
                .body_includes("\"Command\":\"Device/SetDisTempMode\"")
                .body_includes("\"Mode\":0");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, _) = send_json_request(
            &app,
            Method::POST,
            "/manage/weather/temperature-unit/celsius",
            None,
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        mock.assert();
    }

    #[tokio::test]
    async fn temp_unit_sets_fahrenheit() {
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(MockMethod::POST)
                .path("/post")
                .body_includes("\"Command\":\"Device/SetDisTempMode\"")
                .body_includes("\"Mode\":1");
            then.status(200).body(r#"{"error_code":0}"#);
        });

        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, _) = send_json_request(
            &app,
            Method::POST,
            "/manage/weather/temperature-unit/fahrenheit",
            None,
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        mock.assert();
    }

    #[tokio::test]
    async fn temp_unit_rejects_invalid() {
        let server = MockServer::start_async().await;
        let app = build_manage_app(manage_state_with_client(&server.base_url()));
        let (status, body) = send_json_request(
            &app,
            Method::POST,
            "/manage/weather/temperature-unit/kelvin",
            None,
        )
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        let json_body: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json_body["error"], "validation failed");
        assert_eq!(
            json_body["details"]["unit"],
            "unit must be 'celsius' or 'fahrenheit'"
        );
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
        let (status, body) = send_post(
            &app,
            "/manage/display/white-balance",
            Some(json!({ "red": 90, "green": 100, "blue": 100 })),
        )
        .await;

        eprintln!("Status: {status}");
        eprintln!("Body: {body}");

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

        eprintln!("Body: {body}");

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

        eprintln!("missing white balance body: {body}");

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(body.contains("missing"));
    }
}
