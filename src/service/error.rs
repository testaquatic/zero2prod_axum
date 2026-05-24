use crate::database::error::DatabaseError;

#[derive(thiserror::Error, Debug)]
pub enum ServiceError {
    #[error("서버에서 오류가 발생했습니다")]
    DatabaseError(#[from] DatabaseError),
    // 내부 메시지를 표시한다.
    #[error("{0}")]
    ValidationError(String),
}
