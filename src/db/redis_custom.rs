use super::*;
use crate::db::query_builder::{QueryBuilder, QueryOperation};
use crate::utils::errors::{QueryBuilderError, DatabaseError};

pub fn build_query(builder: &QueryBuilder) -> Result<String, QueryBuilderError> {
    match builder.operation {
        QueryOperation::Select => build_get_query(builder),
        QueryOperation::Insert => build_set_query(builder),
        QueryOperation::Update => build_set_query(builder),
        QueryOperation::Delete => build_del_query(builder),
        QueryOperation::Aggregate => Err(QueryBuilderError::InvalidOperation),
    }
}

fn build_get_query(builder: &QueryBuilder) -> Result<String, QueryBuilderError> {
    if builder.table.as_str().is_empty() {
        return Err(QueryBuilderError::MissingField("key".to_string()));
    }
    Ok(format!("GET {}", builder.table.as_str()))
}

fn build_set_query(builder: &QueryBuilder) -> Result<String, QueryBuilderError> {
    if builder.table.as_str().is_empty() || builder.values.is_empty() {
        return Err(QueryBuilderError::MissingField("key or value".to_string()));
    }
    Ok(format!("SET {} {}", builder.table.as_str(), serde_json::to_string(&builder.values[0]).map_err(|e| QueryBuilderError::InvalidQuery(e.to_string()))?))
}

fn build_del_query(builder: &QueryBuilder) -> Result<String, QueryBuilderError> {
    if builder.table.as_str().is_empty() {
        return Err(QueryBuilderError::MissingField("key".to_string()));
    }
    Ok(format!("DEL {}", builder.table.as_str()))
}

pub struct RedisPool {
    client: redis::Client,
}

impl RedisPool {
    pub async fn new(connection_string: &str) -> Result<Self, DatabaseError> {
        let client = redis::Client::open(connection_string)
            .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;
        Ok(RedisPool { client })
    }
}

#[async_trait::async_trait]
impl DatabasePool for RedisPool {
    async fn execute(&self, query: &str, params: Vec<Value>) -> Result<Vec<HashMap<String, Value>>, DatabaseError> {
        let mut con = self.client.get_multiplexed_async_connection().await
            .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;

        let parts: Vec<&str> = query.split_whitespace().collect();
        let command = parts[0];
        let key = parts.get(1).ok_or_else(|| DatabaseError::QueryError("Missing key".to_string()))?;

        let result: redis::RedisResult<String> = match command {
            "GET" => redis::cmd("GET").arg(key).query_async(&mut con).await,
            "SET" => {
                let value = params.get(0).ok_or_else(|| DatabaseError::QueryError("Missing value".to_string()))?;
                redis::cmd("SET").arg(key).arg(serde_json::to_string(value).map_err(|e| DatabaseError::QueryError(e.to_string()))?).query_async(&mut con).await
            },
            "DEL" => redis::cmd("DEL").arg(key).query_async(&mut con).await,
            _ => return Err(DatabaseError::QueryError("Unsupported Redis command".to_string())),
        };

        match result {
            Ok(value) => Ok(vec![
                [(
                    key.to_string(),
                    serde_json::from_str(&value).unwrap_or(Value::String(value))
                )].into_iter().collect()
            ]),
            Err(e) => Err(DatabaseError::QueryError(e.to_string())),
        }
    }
}

#[async_trait::async_trait]
impl Database for RedisPool {
    async fn connect(connection_string: &str) -> Result<Self, DatabaseError> {
        Self::new(connection_string).await
    }

    async fn disconnect(&self) -> Result<(), DatabaseError> {
        // Redis client doesn't require explicit disconnection
        Ok(())
    }

    async fn execute_query(&self, query: &str) -> Result<Vec<Value>, DatabaseError> {
        let mut con = self.client.get_multiplexed_async_connection().await
            .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;

        let result: redis::RedisResult<String> = redis::cmd(query).query_async(&mut con).await;

        match result {
            Ok(value) => Ok(vec![Value::String(value)]),
            Err(e) => Err(DatabaseError::QueryError(e.to_string())),
        }
    }

    async fn list_databases(&self) -> Result<Vec<String>, DatabaseError> {
        // Redis doesn't have a concept of multiple databases in the same way as relational databases
        Ok(vec!["default".to_string()])
    }

    async fn list_collections(&self, _database: &str) -> Result<Vec<String>, DatabaseError> {
        // Redis doesn't have collections, but we can list all keys
        let mut con = self.client.get_multiplexed_async_connection().await
            .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;

        let keys: Vec<String> = redis::cmd("KEYS").arg("*").query_async(&mut con).await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(keys)
    }

    async fn get_schema(&self, _database: &str, _collection: &str) -> Result<Value, DatabaseError> {
        // Redis doesn't have a fixed schema
        Ok(Value::Null)
    }
}