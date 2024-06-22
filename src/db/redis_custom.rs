use redis::AsyncCommands;
use super::*;
use crate::db::query_builder::{QueryBuilder, QueryBuilderError, QueryOperation};

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
    Ok(format!("SET {} {}", builder.table.as_str(), serde_json::to_string(&builder.values[0])?))
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

#[async_trait::async_trait]
impl DatabasePool for RedisPool {
    async fn execute(&self, query: &str, params: Vec<Value>) -> Result<Vec<HashMap<String, Value>>, QueryBuilderError> {
        let mut con = self.client.get_multiplexed_async_connection().await
            .map_err(|e| QueryBuilderError::DatabaseError(e.to_string()))?;

        let parts: Vec<&str> = query.split_whitespace().collect();
        let command = parts[0];
        let key = parts.get(1).ok_or_else(|| QueryBuilderError::InvalidQuery("Missing key".to_string()))?;

        let result: redis::RedisResult<String> = match command {
            "GET" => con.get(key).await,
            "SET" => {
                let value = params.get(0).ok_or_else(|| QueryBuilderError::InvalidQuery("Missing value".to_string()))?;
                con.set(key, serde_json::to_string(value)?).await
            },
            "DEL" => con.del(key).await.map(|n: i64| n.to_string()),
            _ => return Err(QueryBuilderError::InvalidQuery("Unsupported Redis command".to_string())),
        };

        match result {
            Ok(value) => Ok(vec![
                [(
                    key.to_string(),
                    serde_json::from_str(&value).unwrap_or(Value::String(value))
                )].into_iter().collect()
            ]),
            Err(e) => Err(QueryBuilderError::DatabaseError(e.to_string())),
        }
    }
}