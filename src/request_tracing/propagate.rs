use super::request_id::{RequestId, HEADER_NAME};
use axum::{
    body::Body,
    http::{HeaderName, Request},
    middleware::Next,
    response::Response,
};

/// Axum middleware that propagates request IDs through the request lifecycle.
///
/// This middleware:
/// 1. Extracts an existing `X-Request-Id` header from the request, or generates a new UUID
/// 2. Stores the request ID in request extensions (accessible via `Extension<RequestId>`)
/// 3. Records the request ID on the current tracing span
/// 4. Adds the `X-Request-Id` header to the response
///
/// # Panics
///
/// Panics if the header name constant cannot be parsed, which should never happen.
pub async fn propagate(mut req: Request<Body>, next: Next) -> Response {
    let request_id = req
        .headers()
        .get(HEADER_NAME)
        .and_then(RequestId::from_header_value)
        .unwrap_or_default();

    req.extensions_mut().insert(request_id.clone());
    request_id.record();

    let mut response = next.run(req).await;
    let header_name: HeaderName = HEADER_NAME.parse().expect("valid header name");
    response
        .headers_mut()
        .insert(header_name, request_id.header_value());
    response
}
