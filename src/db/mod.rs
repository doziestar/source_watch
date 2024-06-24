/// This module contains the database connection manager, database connection pool, and database connection traits.
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

/// The `DatabasePool` trait defines the methods that a database connection pool should implement.
/// Each database connection pool should implement this trait.
/// Example implementations can be found in the `postgres_custom`, `mongodb_custom`, `redis_custom`, and `elasticsearch_custom` modules.
#[async_trait]
pub trait DatabasePool: Send + Sync {
    async fn execute(&self, query: &str, params: Vec<Value>) -> Result<Vec<HashMap<String, Value>>, DatabaseError>;
}

/// The `Database` trait defines the methods that a database connection should implement.
/// Each database connection should implement this trait.
#[async_trait]
pub trait Database: Send + Sync {
    async fn connect(connection_string: &str) -> Result<Self, DatabaseError> where Self: Sized;
    async fn disconnect(&self) -> Result<(), DatabaseError>;
    async fn execute_query(&self, query: &str) -> Result<Vec<Value>, DatabaseError>;
    async fn list_databases(&self) -> Result<Vec<String>, DatabaseError>;
    async fn list_collections(&self, database: &str) -> Result<Vec<String>, DatabaseError>;
    async fn get_schema(&self, database: &str, collection: &str) -> Result<Value, DatabaseError>;
}

/// The `DatabaseType` enum defines the types of databases that can be used in the application.
/// The `DatabaseManager` uses this enum to identify the type of database connection pool to use.
/// The `DatabaseManager` is used to manage multiple database connection pools.
/// Example usage can be found in the `db_manager` module.
pub mod db_manager {
    use super::*;
    use crate::db::query_builder::{DatabaseType, DatabaseManager};
    use crate::utils::errors::DatabaseError;
    use std::sync::Arc;

    /// Initializes the `DatabaseManager` with the required database connection pools.
    /// This function creates instances of the database connection pools and adds them to the `DatabaseManager`.
    /// The connection strings for the databases should be provided as arguments to the functions.
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