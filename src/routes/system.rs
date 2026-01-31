use axum::{
    extract::Extension,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use pixoo_bridge::pixoo::PixooCommand;
use serde_json::{json, Map, Value};
use std::sync::Arc;
use tracing::{debug, error};

use crate::state::AppState;

pub fn mount_system_routes(router: Router) -> Router {
    router
        .route("/health", get(health))
        .route("/reboot", post(reboot))
}

async fn health(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    if !state.health_forward {
        return (StatusCode::OK, Json(json!({ "status": "ok" })));
    }

    let client = match state.pixoo_client.clone() {
        Some(client) => client,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "status": "unhealthy" })),
            );
        }
    };

    match client.health_check().await {
        Ok(()) => {
            debug!("Pixoo health check succeeded");
            (StatusCode::OK, Json(json!({ "status": "ok" })))
        }
        Err(err) => {
            error!(error = ?err, status = %StatusCode::SERVICE_UNAVAILABLE, "Pixoo health check failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "status": "unhealthy" })),
            )
        }
    }
}

async fn reboot(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    let client = match state.pixoo_client.clone() {
        Some(client) => client,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "error": "Pixoo client unavailable" })),
            )
                .into_response();
        }
    };

    match client
        .send_command(
            PixooCommand::DeviceSysReboot,
            Map::<String, Value>::new(),
        )
        .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => {
            error!(error = ?err, "Pixoo reboot command failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "error": "Pixoo reboot failed" })),
            )
                .into_response()
        }
    }
}
