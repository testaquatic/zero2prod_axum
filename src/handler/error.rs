use axum::response::IntoResponse;

use crate::service::error::ServiceError;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Internal Server Error")]
    InternalError(#[from] ServiceError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::InternalError(error) => {
                tracing::error!("internal server error: {:?}", error);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    error.to_string(),
                )
            }
        };

        (status, message).into_response()
    }
}
