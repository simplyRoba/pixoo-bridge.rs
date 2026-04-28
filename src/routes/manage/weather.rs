use crate::pixoo::client::PixooResponse;
use crate::pixoo::fields::{request as req, response as resp};
use crate::pixoo::PixooCommand;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::sync::Arc;
use tracing::error;
use validator::Validate;

use crate::routes::common::{
    dispatch_pixoo_command, dispatch_pixoo_query, service_unavailable, validation_error_simple,
    ValidatedJson,
};

use super::parsing::{parse_f64, parse_i64, parse_string};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManageWeather {
    weather_string: String,
    current_temperature: f64,
    minimal_temperature: f64,
    maximal_temperature: f64,
    pressure: i64,
    humidity: i64,
    wind_speed: f64,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LocationRequest {
    #[validate(range(min = -180.0, max = 180.0))]
    pub longitude: f64,
    #[validate(range(min = -90.0, max = 90.0))]
    pub latitude: f64,
}

#[tracing::instrument(skip(state))]
pub async fn manage_weather(State(state): State<Arc<AppState>>) -> Response {
    let response = match dispatch_pixoo_query(&state, PixooCommand::ManageGetWeather).await {
        Ok(resp) => resp,
        Err(err) => return err,
    };

    match map_weather(&response) {
        Ok(weather) => axum::Json(weather).into_response(),
        Err(msg) => {
            error!(error = %msg, "failed to map weather response");
            service_unavailable()
        }
    }
}

#[tracing::instrument(skip(state, payload))]
pub async fn manage_set_location(
    State(state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<LocationRequest>,
) -> Response {
    let mut args = Map::new();
    args.insert(
        req::LONGITUDE.to_string(),
        Value::String(payload.longitude.to_string()),
    );
    args.insert(
        req::LATITUDE.to_string(),
        Value::String(payload.latitude.to_string()),
    );

    dispatch_pixoo_command(&state, PixooCommand::ManageSetLocation, args).await
}

#[tracing::instrument(skip(state))]
pub async fn manage_set_temperature_unit(
    State(state): State<Arc<AppState>>,
    Path(unit): Path<String>,
) -> Response {
    let mode_value = match unit.to_ascii_lowercase().as_str() {
        "celsius" => 0,
        "fahrenheit" => 1,
        _ => return validation_error_simple("unit", "unit must be 'celsius' or 'fahrenheit'"),
    };

    let mut args = Map::new();
    args.insert(req::MODE.to_string(), Value::from(mode_value));

    dispatch_pixoo_command(&state, PixooCommand::ManageSetTemperatureUnit, args).await
}

pub fn temperature_unit(response: &PixooResponse) -> Result<String, String> {
    let flag = parse_string(response, resp::TEMPERATURE_MODE)?;
    Ok(if flag == "1" {
        "FAHRENHEIT".to_string()
    } else {
        "CELSIUS".to_string()
    })
}

fn map_weather(response: &PixooResponse) -> Result<ManageWeather, String> {
    Ok(ManageWeather {
        weather_string: parse_string(response, resp::WEATHER)?,
        current_temperature: parse_f64(response, resp::CUR_TEMP)?,
        minimal_temperature: parse_f64(response, resp::MIN_TEMP)?,
        maximal_temperature: parse_f64(response, resp::MAX_TEMP)?,
        pressure: parse_i64(response, resp::PRESSURE)?,
        humidity: parse_i64(response, resp::HUMIDITY)?,
        wind_speed: parse_f64(response, resp::WIND_SPEED)?,
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
}
