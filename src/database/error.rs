#[derive(thiserror::Error, Debug)]
pub enum DatabaseError {
    #[error("Internal Server Error")]
    ServerError(#[from] sqlx::Error),
}
