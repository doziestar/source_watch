pub mod query_builder;
pub mod connection_manager;
pub mod postgres_custom;
pub mod mongodb_custom;
pub mod redis_custom;
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

pub mod db_manager {
    use super::*;
    use crate::db::query_builder::{DatabaseType, DatabaseManager};
    use crate::utils::errors::DatabaseError;
    use std::sync::Arc;

    pub async fn initialize_db_manager() -> Result<DatabaseManager, DatabaseError> {
        let mut db_manager = DatabaseManager::new();

        // Add PostgreSQL pool
        let postgres_pool = Arc::new(postgres_custom::PostgresPool::new("your_postgres_connection_string_here").await?);
        db_manager.add_pool(DatabaseType::PostgreSQL, Box::new(Arc::clone(&postgres_pool) as Arc<dyn DatabasePool>));

        // Add MongoDB pool
        let mongo_pool = Arc::new(mongodb_custom::MongoPool::new("your_mongodb_connection_string_here").await?);
        db_manager.add_pool(DatabaseType::MongoDB, Box::new(Arc::clone(&mongo_pool) as Arc<dyn DatabasePool>));

        // Add Redis pool
        let redis_pool = Arc::new(redis_custom::RedisPool::new("your_redis_connection_string_here").await?);
        db_manager.add_pool(DatabaseType::Redis, Box::new(Arc::clone(&redis_pool) as Arc<dyn DatabasePool>));

        // Add Elasticsearch pool
        let elasticsearch_pool = Arc::new(elasticsearch_custom::ElasticsearchPool::new("your_elasticsearch_connection_string_here").await?);
        db_manager.add_pool(DatabaseType::Elasticsearch, Box::new(Arc::clone(&elasticsearch_pool) as Arc<dyn DatabasePool>));

        Ok(db_manager)
    }
}