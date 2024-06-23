use crate::db::{Database, DatabasePool};
use crate::db::query_builder::{QueryBuilder, QueryOperation, Operator, OrderDirection, Field};
use crate::utils::errors::DatabaseError;
use async_trait::async_trait;
use cassandra_cpp::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use futures::TryFutureExt;

pub struct CassandraPool {
    session: Arc<Session>,
}

impl CassandraPool {
    pub fn new(connection_string: &str) -> Result<Self> {
        let mut cluster = Cluster::default();
        cluster.set_contact_points(connection_string).map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;
        let session = cluster.connect().map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;
        Ok(CassandraPool { session: Arc::new(session) })
    }
}

#[async_trait]
impl Database for CassandraPool {
    async fn connect(connection_string: &str) -> Result<Self> {
        Self::new(connection_string)
    }

    async fn disconnect(&self) -> Result<()> {
        // The session is automatically closed when dropped
        Ok(())
    }

    async fn execute_query(&self, query: &str) -> Result<Vec<Value>> {
        let statement = self.session.prepare(query).map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        let result = statement.execute().map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let rows: Vec<Value> = result.iter().map(|row| {
            let json: serde_json::Map<String, Value> = row.columns().iter().map(|col| {
                (col.name().to_string(), cassandra_value_to_json(col.value()))
            }).collect();
            Value::Object(json)
        }).collect();

        Ok(rows)
    }

    async fn list_databases(&self) -> Result<Vec<String>> {
        let query = "SELECT keyspace_name FROM system_schema.keyspaces;";
        let result = self.execute_query(query).await?;
        Ok(result.into_iter().filter_map(|v| v.get("keyspace_name").and_then(|v| v.as_str()).map(String::from)).collect())
    }

    async fn list_collections(&self, keyspace: &str) -> Result<Vec<String>> {
        let query = format!("SELECT table_name FROM system_schema.tables WHERE keyspace_name = '{}';", keyspace);
        let result = self.execute_query(&query).await?;
        Ok(result.into_iter().filter_map(|v| v.get("table_name").and_then(|v| v.as_str()).map(String::from)).collect())
    }

    async fn get_schema(&self, keyspace: &str, table: &str) -> Result<Value> {
        let query = format!(
            "SELECT column_name, type
             FROM system_schema.columns
             WHERE keyspace_name = '{}' AND table_name = '{}';",
            keyspace, table
        );
        let result = self.execute_query(&query).await?;
        Ok(Value::Array(result))
    }
}

#[async_trait]
impl DatabasePool for CassandraPool {
    async fn execute(&self, query: &str, params: Vec<Value>) -> Result<Vec<HashMap<String, Value>>> {
        let statement = self.session.prepare(query).map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        let values: Vec<Value> = params.into_iter().map(json_to_cassandra_value).collect::<Result<_>>()?;
        let result = statement.execute(&values).map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let rows: Vec<HashMap<String, Value>> = result.iter().map(|row| {
            row.columns().iter().map(|col| {
                (col.name().to_string(), cassandra_value_to_json(col.value()))
            }).collect()
        }).collect();

        Ok(rows)
    }
}

pub fn build_query(builder: &QueryBuilder) -> Result<String> {
    match builder.operation {
        QueryOperation::Select => build_select_query(builder),
        QueryOperation::Insert => build_insert_query(builder),
        QueryOperation::Update => build_update_query(builder),
        QueryOperation::Delete => build_delete_query(builder),
        QueryOperation::Aggregate => Ok("".to_string()), // Not supported
    }
}

fn build_select_query(builder: &QueryBuilder) -> Result<String> {
    let fields = if builder.fields.is_empty() {
        "*".to_string()
    } else {
        builder.fields.iter().map(|f| f.as_str()).collect::<Vec<_>>().join(", ")
    };

    let mut query = format!("SELECT {} FROM {}", fields, builder.table.as_str());

    if !builder.conditions.is_empty() {
        query += " WHERE ";
        query += &builder.conditions.iter()
            .map(|c| format!("{} {} ?", c.field.as_str(), operator_to_cql(&c.operator)))
            .collect::<Vec<_>>().join(" AND ");
    }

    if !builder.order_by.is_empty() {
        query += " ORDER BY ";
        query += &builder.order_by.iter()
            .map(|o| format!("{} {}", o.field.as_str(), if o.direction == OrderDirection::Asc { "ASC" } else { "DESC" }))
            .collect::<Vec<_>>().join(", ");
    }

    if let Some(limit) = builder.limit {
        query += &format!(" LIMIT {}", limit);
    }

    Ok(query)
}

fn build_insert_query(builder: &QueryBuilder) -> Result<String> {
    if builder.fields.is_empty() || builder.values.is_empty() {
        return Err(DatabaseError::MissingField("fields or values".to_string()));
    }

    let fields = builder.fields.iter().map(|f| f.as_str()).collect::<Vec<_>>().join(", ");
    let placeholders = (0..builder.values.len()).map(|_| "?").collect::<Vec<_>>().join(", ");

    Ok(format!("INSERT INTO {} ({}) VALUES ({})", builder.table.as_str(), fields, placeholders))
}

fn build_update_query(builder: &QueryBuilder) -> Result<String> {
    if builder.fields.is_empty() || builder.values.is_empty() {
        return Err(DatabaseError::MissingField("fields or values".to_string()));
    }

    let set_clause = builder.fields.iter()
        .map(|f| format!("{} = ?", f.as_str()))
        .collect::<Vec<_>>().join(", ");

    let mut query = format!("UPDATE {} SET {}", builder.table.as_str(), set_clause);

    if !builder.conditions.is_empty() {
        query += " WHERE ";
        query += &builder.conditions.iter()
            .map(|c| format!("{} {} ?", c.field.as_str(), operator_to_cql(&c.operator)))
            .collect::<Vec<_>>().join(" AND ");
    }

    Ok(query)
}

fn build_delete_query(builder: &QueryBuilder) -> Result<String> {
    let mut query = format!("DELETE FROM {}", builder.table.as_str());

    if !builder.conditions.is_empty() {
        query += " WHERE ";
        query += &builder.conditions.iter()
            .map(|c| format!("{} {} ?", c.field.as_str(), operator_to_cql(&c.operator)))
            .collect::<Vec<_>>().join(" AND ");
    }

    Ok(query)
}

fn operator_to_cql(operator: &Operator) -> &'static str {
    match operator {
        Operator::Eq => "=",
        Operator::Ne => "!=",
        Operator::Gt => ">",
        Operator::Lt => "<",
        Operator::Gte => ">=",
        Operator::Lte => "<=",
        Operator::Like => "LIKE", // Note: Cassandra doesn't support LIKE, you might need to use a different approach
        Operator::In => "IN",
        Operator::NotIn => "NOT IN",
    }
}

fn cassandra_value_to_json(value: &Value) -> Value {
    match value {
        Value::Int(v) => json!(v),
        Value::BigInt(v) => json!(v),
        Value::Float(v) => json!(v),
        Value::Double(v) => json!(v),
        Value::Boolean(v) => json!(v),
        Value::Text(v) => json!(v),
        Value::Blob(v) => json!(v.to_vec()),
        Value::Inet(v) => json!(v.to_string()),
        Value::Uuid(v) => json!(v.to_string()),
        Value::Date(v) => json!(v.to_string()),
        Value::Time(v) => json!(v.to_string()),
        Value::Timestamp(v) => json!(v.to_string()),
        Value::List(v) => json!(v.iter().map(cassandra_value_to_json).collect::<Vec<_>>()),
        Value::Map(v) => json!(v.iter().map(|(k, v)| (k.to_string(), cassandra_value_to_json(v))).collect::<HashMap<_, _>>()),
        Value::Set(v) => json!(v.iter().map(cassandra_value_to_json).collect::<Vec<_>>()),
        Value::Tuple(v) => json!(v.iter().map(cassandra_value_to_json).collect::<Vec<_>>()),
        Value::Udt(v) => json!(v.iter().map(|(k, v)| (k.to_string(), cassandra_value_to_json(v))).collect::<HashMap<_, _>>()),
        _ => Value::Null,
    }
}

fn json_to_cassandra_value(value: Value) -> Result<Value> {
    match value {
        Value::Null => Ok(Value::Null),
        Value::Bool(v) => Ok(Value::Boolean(v)),
        Value::Number(n) => {
            if n.is_i64() {
                Ok(Value::BigInt(n.as_i64().unwrap()))
            } else if n.is_f64() {
                Ok(Value::Double(n.as_f64().unwrap()))
            } else {
                Err(DatabaseError::ConversionError("Unsupported number type".to_string()))
            }
        },
        Value::String(s) => Ok(Value::Text(s)),
        Value::Array(arr) => {
            let values: Result<Vec<_>> = arr.into_iter().map(json_to_cassandra_value).collect();
            Ok(Value::List(values?))
        },
        Value::Object(obj) => {
            let values: Result<HashMap<_, _>> = obj.into_iter()
                .map(|(k, v)| Ok((Value::Text(k), json_to_cassandra_value(v)?)))
                .collect();
            Ok(Value::Map(values?))
        },
    }
}