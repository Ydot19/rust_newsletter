use std::sync::Arc;

use axum::response::Response;
use axum::{body, http, Extension};

fn health_check() -> Response {
    Response::builder()
        .status(http::StatusCode::OK)
        .body(body::Body::from("healthy"))
        .unwrap()
}

pub async fn handler(Extension(_app): axum::Extension<Arc<super::app::Application>>) -> Response {
    health_check()
}
