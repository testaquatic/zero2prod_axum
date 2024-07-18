use std::collections::HashMap;

use axum::{
    extract::Path,
    response::{IntoResponse, Response},
};

// curl -v http://127.0.0.1:8000 => Hello World!
pub async fn root(Path(map): Path<HashMap<String, String>>) -> Response {
    let name = match map.get("name") {
        Some(name) => name,
        None => "World!",
    };
    format!("Hello {}!", name).into_response()
}
