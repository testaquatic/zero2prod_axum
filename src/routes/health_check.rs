use axum::http::StatusCode;

/// /health_check - GET 핸들러
/// 항상 200 OK를 반환한다.
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}
