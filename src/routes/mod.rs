mod draw;
mod manage;
mod system;
mod tools;

use crate::state::AppState;
use axum::Router;
use std::sync::Arc;

/// Mounts all route modules onto the given router.
///
/// When adding a new route module, add it here rather than in main.rs.
pub fn mount_all_routes(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    let router = draw::mount_draw_routes(router);
    let router = tools::mount_tool_routes(router);
    let router = manage::mount_manage_routes(router);
    system::mount_system_routes(router)
}
