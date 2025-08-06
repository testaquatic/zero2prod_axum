use std::collections::HashMap;

use axum::{
    Router,
    extract::{Path, Request},
    routing::get,
};

/// [ axum::extract::path::Path ](<https://jabber-tools.github.io/google_cognitive_apis/doc/0.2.0/axum/extract/path/struct.Path.html>)
/// 위 문서를 참고해서 작성했다.
async fn greet(Path(params): Path<HashMap<String, String>>) -> String {
    let name = params.get("name").cloned().unwrap_or("World".to_owned());
    format!("Hello, {name}!")
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Router::new()
        .route("/", get(greet))
        .route("/{name}", get(greet));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await?;
    axum::serve(listener, app).await
}
