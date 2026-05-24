use axum::response::IntoResponse;

use crate::service::error::ServiceError;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("내부 서버 오류: {0}")]
    InternalError(ServiceError),
    #[error("입력 오류 : {0}")]
    BadRequest(ServiceError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::InternalError(_) => {
                tracing::error!("internal server error: {:?}", self);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    self.to_string(),
                )
            }
            AppError::BadRequest(_) => {
                tracing::error!("validation error: {}", self);
                (axum::http::StatusCode::BAD_REQUEST, self.to_string())
            }
        };

        (status, message).into_response()
    }
}

impl From<ServiceError> for AppError {
    fn from(service_error: ServiceError) -> Self {
        match service_error {
            ServiceError::DatabaseError(_) => AppError::InternalError(service_error),
            ServiceError::ValidationError(_) => AppError::BadRequest(service_error),
        }
    }
}
