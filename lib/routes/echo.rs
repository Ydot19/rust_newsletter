use std::sync::Arc;

use axum::{extract::Query, Extension};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Params {
    name: Option<String>,
}

fn echo(Query(params): Query<Params>) -> String {
    let name = params.name.unwrap_or("world".to_string()).to_string();
    format!("Hello, {}!", name)
}

pub async fn handler(
    Extension(_app): axum::Extension<Arc<super::app::Application>>,
    query: axum::extract::Query<super::echo::Params>,
) -> String {
    echo(query)
}
