use axum::Router;
use axum::routing::{get, post};
use crate::bridge::system::controller;

pub fn define() -> Router {
    Router::new()
        .route("/health/check", get(controller::health_check))
        .route("/reboot", post(controller::reboot))
}