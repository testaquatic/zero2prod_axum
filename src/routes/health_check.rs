use axum::response::{IntoResponse, Response};

// curl -v http://127.0.0.1:8000/health_check => 200 OK
pub async fn health_check() -> Response {
    http::StatusCode::OK.into_response()
}
