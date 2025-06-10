use axum::{
    routing::{get, post},
    Router,
};
use crate::bridge::system;

pub fn define_system_routes() -> Router { 
    Router::new()
        .route("/health/check", get(system::service::health_check))
        .route("/reboot", post(system::service::reboot))
}
 