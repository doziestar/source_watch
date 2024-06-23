use std::fmt;
use cdrs_tokio::error::Error;
use thiserror::Error;

#[derive(Debug)]
pub struct AppError {
    pub message: String,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug, Error)]
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
    #[error("Invalid operation")]
    InvalidOperation,
    #[error("Conversion error: {0}")]
    ConversionError(String),
    #[error("Missing field: {0}")]
    MissingField(String),
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
}


#[derive(Error, Debug)]
pub enum QueryBuilderError {
    #[error("Invalid operation for database type")]
    InvalidOperation,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Unsupported database type")]
    UnsupportedDatabaseType,
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
}