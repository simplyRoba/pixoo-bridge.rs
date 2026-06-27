mod display;
mod time;
mod weather;

use crate::pixoo::client::PixooResponse;
use crate::pixoo::fields::response as resp;
use crate::pixoo::PixooCommand;
use crate::state::AppState;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use std::sync::Arc;
use tracing::error;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::common::{dispatch_pixoo_query, service_unavailable};
use crate::openapi::GenericErrorBody;
use crate::pixoo::error::PixooHttpErrorResponse;

pub fn manage_router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(manage_settings))
        .merge(time::time_router())
        .merge(weather::weather_router())
        .merge(display::display_router())
}

#[derive(Serialize, ToSchema)]
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

#[utoipa::path(
    get,
    path = "/manage/settings",
    tag = "manage",
    responses(
        (status = 200, description = "Current device settings", body = ManageSettings),
        (status = 502, description = "Pixoo device unreachable", body = PixooHttpErrorResponse),
        (status = 503, description = "Pixoo device error (PixooHttpErrorResponse) or unparseable settings (GenericErrorBody)", body = GenericErrorBody),
        (status = 504, description = "Pixoo device timed out", body = PixooHttpErrorResponse)
    )
)]
#[tracing::instrument(skip(state))]
async fn manage_settings(State(state): State<Arc<AppState>>) -> Response {
    let response = match dispatch_pixoo_query(&state, PixooCommand::ManageGetSettings).await {
        Ok(resp) => resp,
        Err(err) => return err,
    };

    match map_settings(&response) {
        Ok(settings) => axum::Json(settings).into_response(),
        Err(msg) => {
            error!(error = %msg, "failed to map settings response");
            service_unavailable()
        }
    }
}

fn map_settings(response: &PixooResponse) -> Result<ManageSettings, String> {
    Ok(ManageSettings {
        display_on: parsing::flag_bool(response, resp::LIGHT_SWITCH)?,
        brightness: parsing::parse_i64(response, resp::BRIGHTNESS)?,
        time_mode: time::time_mode(response)?,
        rotation_angle: rotation_angle(response)?,
        mirrored: parsing::flag_bool(response, resp::MIRROR_FLAG)?,
        temperature_unit: weather::temperature_unit(response)?,
        current_clock_id: parsing::parse_i64(response, resp::CUR_CLOCK_ID)?,
    })
}

fn rotation_angle(response: &PixooResponse) -> Result<i64, String> {
    let flag = parsing::parse_string(response, resp::ROTATION_FLAG)?;
    if flag == "0" {
        return Ok(0);
    }
    let rotation = flag
        .parse::<i64>()
        .map_err(|err| format!("RotationFlag is not integer: {err}"))?;
    Ok(rotation * 90)
}

/// Shared response-parsing helpers used across manage sub-modules.
pub(crate) mod parsing {
    use crate::pixoo::client::PixooResponse;
    use serde_json::Value;

    pub fn parse_string(response: &PixooResponse, key: &str) -> Result<String, String> {
        response
            .get(key)
            .ok_or_else(|| format!("missing field {key}"))
            .and_then(|value| match value {
                Value::String(text) => Ok(text.clone()),
                Value::Number(number) => Ok(number.to_string()),
                other => Err(format!("unexpected type for {key}: {other}")),
            })
    }

    pub fn flag_bool(response: &PixooResponse, key: &str) -> Result<bool, String> {
        Ok(parse_string(response, key)? == "1")
    }

    pub fn parse_i64(response: &PixooResponse, key: &str) -> Result<i64, String> {
        let value = parse_string(response, key)?;
        value
            .parse::<i64>()
            .map_err(|err| format!("{key} is not an integer: {err}"))
    }

    pub fn parse_f64(response: &PixooResponse, key: &str) -> Result<f64, String> {
        let value = parse_string(response, key)?;
        value
            .parse::<f64>()
            .map_err(|err| format!("{key} is not a float: {err}"))
    }
}

#[cfg(test)]
mod tests {
    use super::manage_router;
    use crate::pixoo::{PixooClient, PixooClientConfig};
    use crate::routes::common::testing::send_json_request;
    use crate::state::AppState;
    use axum::http::{Method, StatusCode};
    use axum::Router;
    use httpmock::{Method as MockMethod, MockServer};
    use serde_json::{json, Value};
    use std::sync::Arc;

    fn build_manage_app(state: Arc<AppState>) -> Router {
        let (router, _api) = manage_router().with_state(state).split_for_parts();
        router
    }

    async fn send_get(app: &Router, uri: &str) -> (StatusCode, String) {
        send_json_request(app, Method::GET, uri, None).await
    }

    fn manage_state_with_client(base_url: &str) -> Arc<AppState> {
        let client = PixooClient::new(base_url, PixooClientConfig::default()).expect("client");
        Arc::new(AppState::with_client(client))
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
}
