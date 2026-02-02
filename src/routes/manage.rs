use axum::extract::{Extension, Json};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use chrono::{NaiveDateTime, TimeZone, Utc};
use pixoo_bridge::pixoo::client::PixooResponse;
use pixoo_bridge::pixoo::PixooCommand;
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::sync::Arc;
use tracing::{debug, error};

use crate::state::AppState;

pub fn mount_manage_routes(router: Router) -> Router {
    router
        .route("/manage/settings", get(manage_settings))
        .route("/manage/time", get(manage_time))
        .route("/manage/weather", get(manage_weather))
}

async fn manage_settings(Extension(state): Extension<Arc<AppState>>) -> Response {
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

async fn manage_time(Extension(state): Extension<Arc<AppState>>) -> Response {
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

async fn manage_weather(Extension(state): Extension<Arc<AppState>>) -> Response {
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

async fn dispatch_manage_command(
    state: &AppState,
    command: PixooCommand,
) -> Result<PixooResponse, Response> {
    let client = match state.pixoo_client.clone() {
        Some(client) => client,
        None => return Err(service_unavailable()),
    };

    debug!(%command, "issuing manage command");

    match client.send_command(command.clone(), Map::new()).await {
        Ok(response) => Ok(response),
        Err(err) => {
            error!(command = %command, error = ?err, "Pixoo manage command failed");
            Err(service_unavailable())
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
        .ok_or_else(|| format!("UTCTime {} out of range", utc_secs))?;
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

#[cfg(test)]
mod tests {
    use crate::routes::mount_manage_routes;
    use crate::state::AppState;
    use axum::body::{to_bytes, Body};
    use axum::extract::Extension;
    use axum::http::{Method, Request, StatusCode};
    use axum::Router;
    use chrono::{TimeZone, Utc};
    use httpmock::{Method as MockMethod, MockServer};
    use pixoo_bridge::pixoo::PixooClient;
    use serde_json::json;
    use std::sync::Arc;
    use tower::ServiceExt;

    fn build_manage_app(state: Arc<AppState>) -> Router {
        mount_manage_routes(Router::new()).layer(Extension(state))
    }

    async fn send_get(app: &Router, uri: &str) -> (StatusCode, String) {
        let req = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(req).await.unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap_or_default();
        (status, String::from_utf8_lossy(&body).to_string())
    }

    fn manage_state_with_client(base_url: &str) -> Arc<AppState> {
        let client = PixooClient::new(base_url).expect("client");
        Arc::new(AppState {
            health_forward: false,
            pixoo_client: Some(client),
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
        let json_body: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json_body["displayOn"], true);
        assert_eq!(json_body["brightness"], 80);
        assert_eq!(json_body["timeMode"], "TWENTY_FOUR");
        assert_eq!(json_body["rotationAngle"], 270);
        assert_eq!(json_body["mirrored"], false);
        assert_eq!(json_body["temperatureUnit"], "FAHRENHEIT");
        assert_eq!(json_body["currentClockId"], 5);
    }

    #[tokio::test]
    async fn manage_settings_handles_missing_client() {
        let state = Arc::new(AppState {
            health_forward: false,
            pixoo_client: None,
        });
        let app = build_manage_app(state);
        let (status, body) = send_get(&app, "/manage/settings").await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert!(body.contains("Pixoo command failed"));
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
        let json_body: serde_json::Value = serde_json::from_str(&body).unwrap();
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
        assert!(body.contains("Pixoo command failed"));
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
        let json_body: serde_json::Value = serde_json::from_str(&body).unwrap();
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
        assert!(body.contains("Pixoo command failed"));
    }
}
