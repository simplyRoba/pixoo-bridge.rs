use axum::{
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use axum::http::StatusCode;
use bridge::routes::define_system_routes;

mod bridge;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(hello))
        .route("/json", post(handle_json))
        .merge(define_system_routes())
        .fallback(fallback);
    
    //TODO make port configurable
    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("listening on {}", addr);
    
    axum::serve(listener, app).await.unwrap();
}

async fn hello() -> &'static str {
    "Hello, World!"
}

#[derive(Deserialize)]
struct Input {
    name: String,
}

#[derive(Serialize)]
struct Output {
    message: String,
}

async fn handle_json(Json(payload): Json<Input>) -> Json<Output> {
    Json(Output {
        message: format!("Hello, {}!", payload.name),
    })
}

async fn fallback() -> StatusCode {
    StatusCode::NOT_FOUND
}