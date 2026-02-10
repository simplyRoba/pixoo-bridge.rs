use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::{json, Map, Value};
use validator::{ValidationError, ValidationErrors};

pub fn validation_error_message(error: &ValidationError) -> String {
    if let Some(message) = &error.message {
        message.to_string()
    } else {
        error.code.to_string()
    }
}

pub fn validation_error_response(details: Map<String, Value>) -> Response {
    let body = json!({
        "error": "validation failed",
        "details": Value::Object(details),
    });

    (StatusCode::BAD_REQUEST, Json(body)).into_response()
}

pub fn validation_errors_response(errors: &ValidationErrors) -> Response {
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

pub fn validation_error_simple(field: &str, message: &str) -> Response {
    let mut details = Map::new();
    details.insert(field.to_string(), Value::String(message.to_string()));
    validation_error_response(details)
}

pub fn action_validation_error(action: &str, allowed: &[&str]) -> Response {
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

pub fn service_unavailable() -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({ "error": "Pixoo command failed" })),
    )
        .into_response()
}

pub fn internal_server_error(message: &str) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": message })),
    )
        .into_response()
}
