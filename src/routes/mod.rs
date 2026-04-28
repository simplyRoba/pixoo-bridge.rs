mod common;
mod draw;
mod manage;
mod system;
mod tools;

use axum::http::StatusCode;
use axum::response::Response;
use std::sync::Arc;

use crate::state::AppState;

use axum::Router;

/// Mounts all route modules onto the given router.
///
/// When adding a new route module, add it here rather than in main.rs.
pub fn mount_all_routes(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    let router = draw::mount_draw_routes(router);
    let router = tools::mount_tool_routes(router);
    let router = manage::mount_manage_routes(router);
    system::mount_system_routes(router)
}

/// Returns a JSON 404 response for undefined routes.
pub fn not_found() -> Response {
    common::json_error(StatusCode::NOT_FOUND, "not found").finish()
}
