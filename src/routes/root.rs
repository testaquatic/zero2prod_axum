use std::collections::HashMap;

use axum::{
    extract::Path,
    response::{IntoResponse, Response},
};

// curl -v http://127.0.0.1:8000 => Hello World!
pub async fn root(Path(map): Path<HashMap<String, String>>) -> Response {
    let world = "World".to_string();
    let name = map.get("name").unwrap_or(&world);

    format!("Hello {}!", name).into_response()
}
