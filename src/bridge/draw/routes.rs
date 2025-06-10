use crate::bridge::draw::controller;
use axum::routing::post;
use axum::Router;

pub fn define() -> Router {
    Router::new()
        .route("/fill", post(controller::fill))
}