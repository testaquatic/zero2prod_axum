use axum::{extract::Request, response::IntoResponse};
use reqwest::StatusCode;

use crate::routes::serve_index_file;

/// GET /login을 담당하는 핸들러
pub async fn login_form(request: Request) -> impl IntoResponse {
    serve_index_file(request).await.map_err(|e| {
        tracing::error!("Failed to serve file.\n\tError: {e}");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    })
}
