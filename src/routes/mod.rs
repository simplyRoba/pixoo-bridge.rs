mod common;
mod draw;
mod manage;
mod system;
mod tools;

use axum::http::StatusCode;
use axum::response::Response;
use std::sync::Arc;
use utoipa_axum::router::OpenApiRouter;

use crate::state::AppState;

/// Builds the documented application router by merging every route module.
///
/// Each module returns an [`OpenApiRouter`] so that route registration and
/// `OpenAPI` documentation stay in a single place and cannot drift. When adding
/// a new route module, merge it here rather than in main.rs.
pub fn build_router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .merge(draw::draw_router())
        .merge(tools::tool_router())
        .merge(manage::manage_router())
        .merge(system::system_router())
}

/// Returns a JSON 404 response for undefined routes.
pub fn not_found() -> Response {
    common::json_error(StatusCode::NOT_FOUND, "not found").finish()
}
