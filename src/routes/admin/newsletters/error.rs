use axum::response::{IntoResponse, Response};

use crate::utils::{error_chain_fmt, AppError500};

#[derive(thiserror::Error)]
pub enum AdminPublishError {
    #[error("AdminPublishError: UnexpectedError")]
    UnexpectedError(#[source] anyhow::Error),
}

impl std::fmt::Debug for AdminPublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for AdminPublishError {
    fn into_response(self) -> Response {
        AppError500::new(self).into_response()
    }
}
