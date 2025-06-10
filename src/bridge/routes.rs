use crate::bridge;
use axum::Router;

pub fn bridge_routes() -> Router {
    Router::new()
        .merge(bridge::system::routes::define())
        .nest("/draw", bridge::draw::routes::define())
}
