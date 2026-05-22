#[derive(thiserror::Error, Debug)]
pub enum ServiceError {
    #[error("Internal Server Error")]
    DatabaseError(#[from] sqlx::Error),
}
