use crate::email_client::EmailClientError;

#[derive(thiserror::Error, Debug)]
pub enum ServiceError {
    #[error("서버에서 오류가 발생했습니다")]
    DatabaseError(#[from] sqlx::Error),
    #[error("서버에서 오류가 발생했습니다")]
    SendEmailError(#[from] EmailClientError),
    // 내부 메시지를 표시한다.
    #[error("{0}")]
    ValidationError(String),
    #[error("서버에서 오류가 발생했습니다")]
    SubscriptionTokenError,
    #[error("사용자 정보를 찾을 수 없습니다")]
    AuthError(#[source] anyhow::Error),
    #[error("서버에서 오류가 발생했습니다")]
    UnexpectedError(#[source] anyhow::Error),
}
