use crate::database::error::DatabaseError;

#[derive(thiserror::Error, Debug)]
pub enum ServiceError {
    #[error("Internal Server Error")]
    DatabaseError(#[from] DatabaseError),
    #[error("Invalid Input")]
    ValidationError(String),
}
