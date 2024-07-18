use std::collections::HashMap;

use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Router,
};

// curl -v http://127.0.0.1:8000 => Hello World!
// curl -v http://127.0.0.1:8000/wow => Hello wow!
async fn greet(Path(map): Path<HashMap<String, String>>) -> Response {
    let world = "World".to_string();
    let name = map.get("name").unwrap_or(&world);

    format!("Hello {}!", name).into_response()
}

// curl -v http://127.0.0.1:8000/health_check => 200 OK
async fn health_check() -> Response {
    StatusCode::OK.into_response()
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Router::new()
        .route("/", routing::get(greet))
        .route("/health_check", routing::get(health_check));
    let tcp_listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await?;
    axum::serve(tcp_listener, app).await
}
