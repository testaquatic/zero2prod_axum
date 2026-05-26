use axum::{
    http::{self},
    response::IntoResponse,
};

use crate::service::error::ServiceError;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("내부 서버 오류: {0}")]
    InternalError(#[source] ServiceError),
    #[error("입력 오류 : {0}")]
    BadRequest(#[source] ServiceError),
    #[error("인증 오류")]
    AuthError(#[source] anyhow::Error),
    #[error("서버 오류: 예상하지 못한 오류")]
    UnexpectedError(#[source] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        // 로깅
        match &self {
            AppError::InternalError(_) | AppError::UnexpectedError(_) => {
                tracing::error!("internal server error: {}", self);
            }
            AppError::BadRequest(_) => {
                tracing::error!("validation error: {}", self);
            }
            AppError::AuthError(_) => {
                tracing::warn!("unauthorized: {}", self);
            }
        }

        // 응답
        match self {
            AppError::InternalError(_) | AppError::UnexpectedError(_) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                self.to_string(),
            )
                .into_response(),
            AppError::BadRequest(_) => {
                (axum::http::StatusCode::BAD_REQUEST, self.to_string()).into_response()
            }
            AppError::AuthError(_) => (
                axum::http::StatusCode::UNAUTHORIZED,
                [(http::header::WWW_AUTHENTICATE, r#"Basic realm="publish""#)],
                self.to_string(),
            )
                .into_response(),
        }
    }
}

impl From<ServiceError> for AppError {
    fn from(service_error: ServiceError) -> Self {
        match service_error {
            ServiceError::DatabaseError(_)
            | ServiceError::SendEmailError(_)
            | ServiceError::SubscriptionTokenError
            | ServiceError::UnexpectedError(_) => AppError::InternalError(service_error),
            ServiceError::ValidationError(_) => AppError::BadRequest(service_error),
            ServiceError::AuthError => AppError::AuthError(anyhow::anyhow!(service_error)),
        }
    }
}
