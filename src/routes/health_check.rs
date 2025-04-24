use axum::response::IntoResponse;

/// /health_check 핸들러이다.
pub async fn health_check() -> impl IntoResponse {
    axum::http::StatusCode::OK
}
