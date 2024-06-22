use super::*;
use cdrs_tokio::query::QueryValues;
use cdrs_tokio::types::prelude::*;
use serde_json::json;
use crate::db::query_builder::{Operator, OrderDirection, QueryBuilder, QueryBuilderError, QueryOperation};

pub fn build_query(builder: &QueryBuilder) -> Result<String> {
    match builder.operation {
        QueryOperation::Select => build_select_query(builder),
        QueryOperation::Insert => build_insert_query(builder),
        QueryOperation::Update => build_update_query(builder),
        QueryOperation::Delete => build_delete_query(builder),
        QueryOperation::Aggregate => Err(QueryBuilderError::InvalidOperation),
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
        return Err(QueryBuilderError::MissingField("fields or values".to_string()));
    }

    let fields = builder.fields.iter().map(|f| f.as_str()).collect::<Vec<_>>().join(", ");
    let placeholders = (0..builder.values.len()).map(|_| "?").collect::<Vec<_>>().join(", ");

    Ok(format!("INSERT INTO {} ({}) VALUES ({})", builder.table.as_str(), fields, placeholders))
}

fn build_update_query(builder: &QueryBuilder) -> Result<String> {
    if builder.fields.is_empty() || builder.values.is_empty() {
        return Err(QueryBuilderError::MissingField("fields or values".to_string()));
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

fn build_delete_query(builder: &QueryBuilder) -> Result<String, QueryBuilderError> {
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
        Operator::Like => "LIKE",
        Operator::In => "IN",
        Operator::NotIn => "NOT IN",
    }
}

pub struct CassandraPool {
    session: cdrs_tokio::Session<cdrs_tokio::transport::TransportTcp>,
}

#[async_trait::async_trait]
impl DatabasePool for CassandraPool {
    async fn execute(&self, query: &str, params: Vec<Value>) -> Result<Vec<HashMap<String, Value>>> {
        let values: Vec<_> = params.into_iter().map(|v| v.into()).collect();
        let query_values = QueryValues::SimpleValues(values);

        let rows = self.session.query_with_values(query, query_values).await
            .map_err(|e| QueryBuilderError::DatabaseError(e.to_string()))?
            .rows
            .ok_or_else(|| QueryBuilderError::DatabaseError("No rows returned".to_string()))?;

        Ok(rows.into_iter().map(|row| {
            row.into_iter().map(|(column_name, column_value)| {
                (column_name, cassandra_value_to_json(column_value))
            }).collect()
        }).collect())
    }
}

fn cassandra_value_to_json(value: cdrs_tokio::types::value::Value) -> Value {
    match value {
        Value::BigInt(v) => json!(v),
        Value::Blob(v) => json!(v),
        Value::Boolean(v) => json!(v),
        Value::Counter(v) => json!(v),
        Value::Decimal(v) => json!(v.to_string()),
        Value::Double(v) => json!(v),
        Value::Float(v) => json!(v),
        Value::Int(v) => json!(v),
        Value::Text(v) => json!(v),
        Value::Timestamp(v) => json!(v.to_string()),
        Value::Uuid(v) => json!(v.to_string()),
        Value::Varchar(v) => json!(v),
        Value::Varint(v) => json!(v.to_string()),
        Value::Timeuuid(v) => json!(v.to_string()),
        Value::Inet(v) => json!(v.to_string()),
        Value::Date(v) => json!(v.to_string()),
        Value::Time(v) => json!(v.to_string()),
        Value::SmallInt(v) => json!(v),
        Value::TinyInt(v) => json!(v),
        Value::List(v) => json!(v.into_iter().map(cassandra_value_to_json).collect::<Vec<_>>()),
        Value::Map(v) => json!(v.into_iter().map(|(k, v)| (k, cassandra_value_to_json(v))).collect::<HashMap<_, _>>()),
        Value::Set(v) => json!(v.into_iter().map(cassandra_value_to_json).collect::<Vec<_>>()),
        Value::Udt(v) => json!(v.into_iter().map(|(k, v)| (k, cassandra_value_to_json(v))).collect::<HashMap<_, _>>()),
        Value::Tuple(v) => json!(v.into_iter().map(cassandra_value_to_json).collect::<Vec<_>>()),
        _ => Value::Null,
    }
}