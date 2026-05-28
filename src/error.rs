use axum::{Json, response::IntoResponse};

use crate::{domain::response::AppErrorMessage, service::error::ServiceError};

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("서버 오류: 서버에서 오류가 발생했습니다")]
    InternalError(#[source] ServiceError),
    #[error("입력 오류: {0}")]
    BadRequest(#[source] ServiceError),
    #[error("인증 오류: 사용자 정보를 찾을 수 없습니다")]
    AuthError(#[source] anyhow::Error),
    #[error("서버 오류: 예상하지 못한 오류가 방생했습니다")]
    UnexpectedError(#[source] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        // 로깅
        match &self {
            AppError::InternalError(_) | AppError::UnexpectedError(_) => {
                tracing::error!("internal server error: {:?}", self);
            }
            AppError::BadRequest(_) => {
                tracing::error!("validation error: {:?}", self);
            }
            AppError::AuthError(_) => {
                tracing::warn!("unauthorized: {:?}", self);
            }
        }

        // 응답
        let app_error_message = match self {
            AppError::InternalError(_) | AppError::UnexpectedError(_) => AppErrorMessage {
                status: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                message: self.to_string(),
            },
            AppError::BadRequest(_) => AppErrorMessage {
                status: axum::http::StatusCode::BAD_REQUEST,
                message: self.to_string(),
            },
            AppError::AuthError(_) => AppErrorMessage {
                status: axum::http::StatusCode::UNAUTHORIZED,
                message: self.to_string(),
            },
        };

        (app_error_message.status, Json(app_error_message)).into_response()
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
            ServiceError::AuthError(_) => AppError::AuthError(anyhow::anyhow!(service_error)),
        }
    }
}
