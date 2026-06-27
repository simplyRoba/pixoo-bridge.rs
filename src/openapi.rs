//! Top-level `OpenAPI` document and shared error schemas.
//!
//! The concrete paths are collected from the live router via `utoipa-axum`
//! (see `crate::routes::build_router`), so this module only defines the
//! document metadata and the error response schemas that handlers reference.

use serde::Serialize;
use utoipa::{OpenApi, ToSchema};

use crate::pixoo::error::{PixooHttpErrorKind, PixooHttpErrorResponse};

/// Base `OpenAPI` document. Paths and request/response schemas are merged in at
/// runtime from the `OpenApiRouter`; only metadata lives here.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Pixoo Bridge",
        description = "HTTP bridge for controlling a Divoom Pixoo matrix.",
        version = env!("CARGO_PKG_VERSION")
    ),
    tags(
        (name = "draw", description = "Drawing pixels, images, and text"),
        (name = "tools", description = "Built-in Pixoo tools (timer, stopwatch, scoreboard, sound meter)"),
        (name = "manage", description = "Device settings: display, time, and weather"),
        (name = "system", description = "Health and system control")
    ),
    components(schemas(
        PixooHttpErrorResponse,
        PixooHttpErrorKind,
        ValidationErrorBody,
        PayloadTooLargeBody
    ))
)]
pub struct ApiDoc;

/// `400 Bad Request` body returned when payload or path validation fails.
#[derive(Serialize, ToSchema)]
#[allow(dead_code)]
pub struct ValidationErrorBody {
    /// Always `"validation failed"`.
    #[schema(example = "validation failed")]
    pub error: String,
    /// Field- or action-specific details. Shape depends on the failing input.
    #[schema(value_type = Object, example = json!({ "red": ["range"] }))]
    pub details: serde_json::Value,
}

/// `413 Payload Too Large` body returned when an image exceeds the size limit.
#[derive(Serialize, ToSchema)]
#[allow(dead_code)]
pub struct PayloadTooLargeBody {
    /// Always `"file too large"`.
    #[schema(example = "file too large")]
    pub error: String,
    /// Configured maximum size in bytes.
    #[schema(example = 5_242_880)]
    pub limit: usize,
    /// Actual payload size in bytes.
    #[schema(example = 6_000_000)]
    pub actual: usize,
}
