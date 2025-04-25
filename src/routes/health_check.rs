use axum::{http::StatusCode, response::IntoResponse};

/// /health_check 핸들러이다.
pub async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}
