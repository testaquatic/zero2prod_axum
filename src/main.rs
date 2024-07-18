use std::collections::HashMap;

use axum::{
    extract::Path,
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

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Router::new()
        .route("/", routing::get(greet))
        .route("/:name", routing::get(greet));
    let tcp_listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await?;
    axum::serve(tcp_listener, app).await
}
