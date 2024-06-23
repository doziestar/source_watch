use crate::db::{
    Database, DatabasePool,
    query_builder::{QueryBuilder, QueryOperation, Operator, OrderDirection, Field}
};
use crate::utils::errors::{DatabaseError, QueryBuilderError};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio_postgres;
use deadpool_postgres;
use tokio_postgres::types::Type;

pub fn build_query(builder: &QueryBuilder) -> Result<String, QueryBuilderError> {
    let mut query = match builder.operation {
        QueryOperation::Select => {
            let fields = if builder.fields.is_empty() {
                "*".to_string()
            } else {
                builder.fields.iter().map(|f| f.as_str()).collect::<Vec<_>>().join(", ")
            };
            format!("SELECT {} FROM {}", fields, builder.table.as_str())
        }
        QueryOperation::Insert => {
            if builder.fields.is_empty() || builder.values.is_empty() {
                return Err(QueryBuilderError::MissingField("fields or values".to_string()));
            }
            let fields = builder.fields.iter().map(|f| f.as_str()).collect::<Vec<_>>().join(", ");
            let placeholders = (1..=builder.values.len()).map(|i| format!("${}", i)).collect::<Vec<_>>().join(", ");
            format!("INSERT INTO {} ({}) VALUES ({})", builder.table.as_str(), fields, placeholders)
        }
        QueryOperation::Update => {
            if builder.fields.is_empty() || builder.values.is_empty() {
                return Err(QueryBuilderError::MissingField("fields or values".to_string()));
            }
            let set_clause = builder.fields.iter().enumerate()
                .map(|(i, f)| format!("{} = ${}", f.as_str(), i + 1))
                .collect::<Vec<_>>().join(", ");
            format!("UPDATE {} SET {}", builder.table.as_str(), set_clause)
        }
        QueryOperation::Delete => format!("DELETE FROM {}", builder.table.as_str()),
        QueryOperation::Aggregate => return Err(QueryBuilderError::InvalidOperation),
    };

    if !builder.conditions.is_empty() {
        query += " WHERE ";
        query += &builder.conditions.iter().enumerate()
            .map(|(i, c)| format!("{} {} ${}", c.field.as_str(), operator_to_sql(&c.operator), builder.values.len() + i + 1))
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

    if let Some(offset) = builder.offset {
        query += &format!(" OFFSET {}", offset);
    }

    Ok(query)
}

fn operator_to_sql(operator: &Operator) -> &'static str {
    match operator {
        Operator::Eq => "=",
        Operator::Ne => "!=",
        Operator::Gt => ">",
        Operator::Lt => "<",
        Operator::Gte => ">=",
        Operator::Lte => "<=",
        Operator::Like => "LIKE",
        Operator::In => "IN",
        Operator::NotIn => "NOT IN",
    }
}

pub struct PostgresPool {
    pool: deadpool_postgres::Pool,
}

impl PostgresPool {
    pub async fn new(connection_string: &str) -> Result<Self, DatabaseError> {
        let config = connection_string.parse()
            .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;
        let pool = deadpool_postgres::Pool::new(config);
        Ok(PostgresPool { pool })
    }
}

#[async_trait]
impl Database for PostgresPool {
    async fn connect(connection_string: &str) -> Result<Self, DatabaseError> {
        Self::new(connection_string).await
    }

    async fn disconnect(&self) -> Result<(), DatabaseError> {
        // Deadpool handles connection cleanup automatically
        Ok(())
    }

    async fn execute_query(&self, query: &str) -> Result<Vec<Value>, DatabaseError> {
        let client = self.pool.get().await
            .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;
        let rows = client.query(query, &[]).await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(rows.into_iter().map(|row| {
            let json: serde_json::Map<String, Value> = row.columns()
                .iter()
                .map(|column| (column.name().to_string(), postgres_value_to_json(&row, column)))
                .collect();
            Value::Object(json)
        }).collect())
    }

    async fn list_databases(&self) -> Result<Vec<String>, DatabaseError> {
        let query = "SELECT datname FROM pg_database WHERE datistemplate = false;";
        let result = self.execute_query(query).await?;
        Ok(result.into_iter().filter_map(|v| v.get("datname").and_then(|v| v.as_str()).map(String::from)).collect())
    }

    async fn list_collections(&self, database: &str) -> Result<Vec<String>, DatabaseError> {
        let query = format!("SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_catalog = '{}';", database);
        let result = self.execute_query(&query).await?;
        Ok(result.into_iter().filter_map(|v| v.get("table_name").and_then(|v| v.as_str()).map(String::from)).collect())
    }

    async fn get_schema(&self, database: &str, table: &str) -> Result<Value, DatabaseError> {
        let query = format!(
            "SELECT column_name, data_type, is_nullable
             FROM information_schema.columns
             WHERE table_schema = 'public'
               AND table_catalog = '{}'
               AND table_name = '{}';",
            database, table
        );
        let result = self.execute_query(&query).await?;
        Ok(Value::Array(result))
    }
}

#[async_trait]
impl DatabasePool for PostgresPool {
    async fn execute(&self, query: &str, params: Vec<Value>) -> Result<Vec<HashMap<String, Value>>, DatabaseError> {
        let client = self.pool.get().await
            .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;
        let stmt = client.prepare(query).await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params.iter()
            .map(|v| v as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let rows = client.query(&stmt, &params)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(rows.into_iter().map(|row| {
            row.columns().iter().map(|column| {
                (column.name().to_string(), postgres_value_to_json(&row, column))
            }).collect()
        }).collect())
    }
}

fn postgres_value_to_json(row: &tokio_postgres::Row, column: &tokio_postgres::Column) -> Value {
    let col_idx = column.ordinal();
    match column.type_() {
        &Type::BOOL => json!(row.try_get::<_, Option<bool>>(col_idx).unwrap_or(None)),
        &Type::INT2 => json!(row.try_get::<_, Option<i16>>(col_idx).unwrap_or(None)),
        &Type::INT4 => json!(row.try_get::<_, Option<i32>>(col_idx).unwrap_or(None)),
        &Type::INT8 => json!(row.try_get::<_, Option<i64>>(col_idx).unwrap_or(None)),
        &Type::FLOAT4 => json!(row.try_get::<_, Option<f32>>(col_idx).unwrap_or(None)),
        &Type::FLOAT8 => json!(row.try_get::<_, Option<f64>>(col_idx).unwrap_or(None)),
        &Type::VARCHAR | &Type::TEXT | &Type::BPCHAR => json!(row.try_get::<_, Option<String>>(col_idx).unwrap_or(None)),
        &Type::JSON | &Type::JSONB => {
            match row.try_get::<_, Option<serde_json::Value>>(col_idx) {
                Ok(Some(v)) => v,
                _ => Value::Null,
            }
        },
        &Type::TIMESTAMP => {
            let dt: Option<chrono::NaiveDateTime> = row.try_get(col_idx).unwrap_or(None);
            json!(dt.map(|d| d.to_string()))
        },
        &Type::TIMESTAMPTZ => {
            let dt: Option<chrono::DateTime<chrono::Utc>> = row.try_get(col_idx).unwrap_or(None);
            json!(dt.map(|d| d.to_rfc3339()))
        },
        &Type::DATE => {
            let d: Option<chrono::NaiveDate> = row.try_get(col_idx).unwrap_or(None);
            json!(d.map(|d| d.to_string()))
        },
        &Type::TIME => {
            let t: Option<chrono::NaiveTime> = row.try_get(col_idx).unwrap_or(None);
            json!(t.map(|t| t.to_string()))
        },
        _ => Value::Null,
    }
}