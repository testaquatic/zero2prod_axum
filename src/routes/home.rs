use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse};

use crate::startup::IndexHtml;

/// /home GET을 담당하는 핸들러이다.
pub async fn home(State(index_html): State<Arc<IndexHtml>>) -> impl IntoResponse {
    (StatusCode::OK, index_html.pub_html.to_string()).into_response()
}
