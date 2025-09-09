use std::sync::Arc;

use axum::{
    extract::State,
    http::header,
    response::{ErrorResponse, IntoResponse, Response},
};

use crate::startup::IndexHtml;

pub async fn change_password_form(
    State(index_html): State<Arc<IndexHtml>>,
) -> Result<Response, ErrorResponse> {
    let response = (
        [(header::CONTENT_TYPE, "text/html")],
        index_html.admin_html.clone(),
    )
        .into_response();

    Ok(response)
}
