use axum::body::Body;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, Json, Request};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::de::DeserializeOwned;
use serde_json::{json, Map, Value};
use validator::{Validate, ValidationError, ValidationErrors};

/// A JSON extractor that deserializes and validates the request body,
/// returning consistent validation error responses on failure.
pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|err: JsonRejection| validation_error_simple("body", &err.body_text()))?;

        if let Err(errors) = value.validate() {
            return Err(validation_errors_response(&errors));
        }

        Ok(ValidatedJson(value))
    }
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

#[cfg(test)]
pub mod testing {
    use axum::body::{to_bytes, Body};
    use axum::http::{Method, Request, StatusCode};
    use axum::Router;
    use tower::util::ServiceExt;

    pub async fn send_json_request(
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
}
