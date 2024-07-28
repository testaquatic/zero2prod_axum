use axum::response::{IntoResponse, Response};

// 뉴스레터 발송을 담당하는 엔드포인트
pub async fn publish_newsletter() -> Response {
    http::StatusCode::OK.into_response()
}
