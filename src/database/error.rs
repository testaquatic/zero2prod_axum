#[derive(thiserror::Error, Debug)]
pub enum DatabaseError {
    #[error("서버에서 오류가 발생했습니다")]
    ServerError(#[from] sqlx::Error),
}
