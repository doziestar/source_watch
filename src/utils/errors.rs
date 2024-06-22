use std::fmt;

#[derive(Debug)]
pub struct AppError {
    pub message: String,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AppError {}

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Query execution error: {0}")]
    QueryError(String),
    #[error("Database not found: {0}")]
    DatabaseNotFound(String),
    #[error("Collection not found: {0}")]
    CollectionNotFound(String),
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
}