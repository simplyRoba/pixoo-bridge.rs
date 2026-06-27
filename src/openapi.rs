//! Top-level `OpenAPI` document and shared error schemas.
//!
//! The concrete paths are collected from the live router via `utoipa-axum`
//! (see `crate::routes::build_router`), so this module only defines the
//! document metadata and the error response schemas that handlers reference.

use utoipa::OpenApi;

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
    components(schemas(PixooHttpErrorResponse, PixooHttpErrorKind))
)]
pub struct ApiDoc;
