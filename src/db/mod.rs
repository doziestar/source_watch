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

#[async_trait]
pub trait DatabasePool: Send + Sync {
    async fn execute(&self, query: &str, params: Vec<Value>) -> Result<Vec<HashMap<String, Value>>, query_builder::QueryBuilderError>;
}