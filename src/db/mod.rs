pub mod query_builder;
pub mod connection_manager;
pub mod postgres_custom;
pub mod mongodb_custom;
pub mod redis_custom;
pub mod cassandra;
pub mod elasticsearch_custom;

use serde_json::Value;
use std::collections::HashMap;
use async_trait::async_trait;
use crate::utils::errors::DatabaseError;

#[async_trait]
pub trait DatabasePool: Send + Sync {
    async fn execute(&self, query: &str, params: Vec<Value>) -> Result<Vec<HashMap<String, Value>>, DatabaseError>;
}

#[async_trait]
pub trait Database: Send + Sync {
    async fn connect(connection_string: &str) -> Result<Self, DatabaseError> where Self: Sized;
    async fn disconnect(&self) -> Result<(), DatabaseError>;
    async fn execute_query(&self, query: &str) -> Result<Vec<Value>, DatabaseError>;
    async fn list_databases(&self) -> Result<Vec<String>, DatabaseError>;
    async fn list_collections(&self, database: &str) -> Result<Vec<String>, DatabaseError>;
    async fn get_schema(&self, database: &str, collection: &str) -> Result<Value, DatabaseError>;
}