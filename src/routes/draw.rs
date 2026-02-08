use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use pixoo_bridge::pixoo::client::PixooResponse;
use pixoo_bridge::pixoo::{map_pixoo_error, PixooClient, PixooCommand};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::sync::Arc;
use tracing::error;
use validator::{Validate, ValidationError, ValidationErrors};

use crate::draw::{encode_pic_data, uniform_pixel_buffer};
use crate::state::AppState;

const SINGLE_FRAME_PIC_SPEED_MS: i64 = 9999;

pub fn mount_draw_routes(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router.route("/draw/fill", post(draw_fill))
}

#[derive(Debug, Deserialize, Validate)]
struct DrawFillRequest {
    #[validate(range(min = 0, max = 255))]
    red: u16,
    #[validate(range(min = 0, max = 255))]
    green: u16,
    #[validate(range(min = 0, max = 255))]
    blue: u16,
}

async fn draw_fill(State(state): State<Arc<AppState>>, Json(payload): Json<Value>) -> Response {
    let payload = match serde_json::from_value::<DrawFillRequest>(payload) {
        Ok(request) => request,
        Err(err) => {
            let message = err.to_string();
            return validation_error_simple("body", &message);
        }
    };

    if let Err(errors) = payload.validate() {
        return validation_errors_response(&errors);
    }

    let Some(client) = state.pixoo_client.clone() else {
        return service_unavailable();
    };

    let Ok(red) = u8::try_from(payload.red) else {
        return internal_server_error("invalid red value");
    };
    let Ok(green) = u8::try_from(payload.green) else {
        return internal_server_error("invalid green value");
    };
    let Ok(blue) = u8::try_from(payload.blue) else {
        return internal_server_error("invalid blue value");
    };

    let buffer = uniform_pixel_buffer(red, green, blue);
    let pic_data = match encode_pic_data(&buffer) {
        Ok(value) => value,
        Err(err) => {
            error!(error = %err, "failed to encode draw payload");
            return internal_server_error("failed to encode draw payload");
        }
    };

    let response = match client
        .send_command(PixooCommand::DrawGetGifId, Map::new())
        .await
    {
        Ok(response) => response,
        Err(err) => {
            let (status, body) = map_pixoo_error(&err, "Pixoo draw id command");
            error!(error = ?err, status = %status, "Pixoo draw id command failed");
            return (status, body).into_response();
        }
    };

    let pic_id = match parse_pic_id(&response) {
        Ok(value) => value,
        Err(err) => {
            error!(error = %err, response = ?response, "missing PicID in draw response");
            return service_unavailable();
        }
    };

    send_draw_gif(&client, pic_id, 1, 0, pic_data).await
}

fn parse_pic_id(response: &PixooResponse) -> Result<i64, String> {
    let value = response
        .get("PicID")
        .ok_or_else(|| "missing PicID".to_string())?;
    match value {
        Value::Number(number) => number
            .as_i64()
            .or_else(|| number.as_u64().and_then(|v| i64::try_from(v).ok()))
            .ok_or_else(|| "PicID is not an integer".to_string()),
        Value::String(text) => text
            .parse::<i64>()
            .map_err(|_| "PicID is not an integer".to_string()),
        _ => Err("PicID is not an integer".to_string()),
    }
}

async fn send_draw_gif(
    client: &PixooClient,
    pic_id: i64,
    pic_num: i64,
    pic_offset: i64,
    pic_data: String,
) -> Response {
    let mut args = Map::new();
    args.insert("PicID".to_string(), Value::from(pic_id));
    args.insert("PicNum".to_string(), Value::from(pic_num));
    args.insert("PicOffset".to_string(), Value::from(pic_offset));
    args.insert("PicWidth".to_string(), Value::from(64));
    args.insert(
        "PicSpeed".to_string(),
        Value::from(SINGLE_FRAME_PIC_SPEED_MS),
    );
    args.insert("PicData".to_string(), Value::String(pic_data));

    match client.send_command(PixooCommand::DrawSendGif, args).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => {
            let (status, body) = map_pixoo_error(&err, "Pixoo draw send command");
            error!(error = ?err, status = %status, "Pixoo draw send command failed");
            (status, body).into_response()
        }
    }
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

fn validation_error_simple(field: &str, message: &str) -> Response {
    let mut details = Map::new();
    details.insert(field.to_string(), Value::String(message.to_string()));
    validation_error_response(details)
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

#[cfg(test)]
mod tests {
    use super::SINGLE_FRAME_PIC_SPEED_MS;
    use crate::draw::{encode_pic_data, uniform_pixel_buffer};
    use crate::routes::mount_draw_routes;
    use crate::state::AppState;
    use axum::body::{to_bytes, Body};
    use axum::extract::State as AxumState;
    use axum::http::{Method, Request, StatusCode};
    use axum::routing::post as axum_post;
    use axum::{Json, Router};
    use pixoo_bridge::pixoo::PixooClient;
    use serde_json::{json, Value};
    use std::sync::{Arc, Mutex};
    use tokio::net::TcpListener;
    use tower::ServiceExt;

    #[derive(Clone)]
    struct PixooMockState {
        requests: Arc<Mutex<Vec<Value>>>,
    }

    async fn pixoo_mock_handler(
        AxumState(state): AxumState<PixooMockState>,
        Json(body): Json<Value>,
    ) -> (StatusCode, Json<Value>) {
        state.requests.lock().unwrap().push(body.clone());
        let command = body
            .get("Command")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if command == "Draw/GetHttpGifId" {
            (
                StatusCode::OK,
                Json(json!({ "error_code": 0, "PicID": 42 })),
            )
        } else {
            (StatusCode::OK, Json(json!({ "error_code": 0 })))
        }
    }

    async fn start_pixoo_mock() -> (String, Arc<Mutex<Vec<Value>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let state = PixooMockState {
            requests: requests.clone(),
        };
        let app = Router::new()
            .route("/post", axum_post(pixoo_mock_handler))
            .with_state(state);
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
        let addr = listener.local_addr().expect("addr");
        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("server");
        });
        (format!("http://{addr}"), requests)
    }

    fn build_draw_app(state: Arc<AppState>) -> Router {
        mount_draw_routes(Router::new()).with_state(state)
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

    #[tokio::test]
    async fn draw_fill_sends_expected_command_sequence() {
        let (base_url, requests) = start_pixoo_mock().await;
        let client = PixooClient::new(base_url).expect("client");
        let state = Arc::new(AppState {
            health_forward: false,
            pixoo_client: Some(client),
        });
        let app = build_draw_app(state);

        let (status, body) = send_json_request(
            &app,
            Method::POST,
            "/draw/fill",
            Some(json!({ "red": 32, "green": 128, "blue": 16 })),
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        assert!(body.is_empty());

        let captured = requests.lock().unwrap();
        assert_eq!(captured.len(), 2);
        assert_eq!(captured[0]["Command"], "Draw/GetHttpGifId");
        assert_eq!(captured[1]["Command"], "Draw/SendHttpGif");
        assert_eq!(captured[1]["PicID"], 42);
        assert_eq!(captured[1]["PicNum"], 1);
        assert_eq!(captured[1]["PicOffset"], 0);
        assert_eq!(captured[1]["PicWidth"], 64);
        assert_eq!(captured[1]["PicSpeed"], SINGLE_FRAME_PIC_SPEED_MS);
        let expected_buffer = uniform_pixel_buffer(32, 128, 16);
        let expected_pic_data = encode_pic_data(&expected_buffer).expect("picdata");
        assert_eq!(captured[1]["PicData"], expected_pic_data);
    }

    #[tokio::test]
    async fn draw_fill_rejects_invalid_payload() {
        let state = Arc::new(AppState {
            health_forward: false,
            pixoo_client: None,
        });
        let app = build_draw_app(state);
        let (status, body) = send_json_request(
            &app,
            Method::POST,
            "/draw/fill",
            Some(json!({ "red": 999, "green": 0, "blue": 0 })),
        )
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        let json_body: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json_body["error"], "validation failed");
        assert!(json_body["details"]["red"].is_array());
    }
}
