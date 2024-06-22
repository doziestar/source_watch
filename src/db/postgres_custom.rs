use super::*;
use crate::db::query_builder::{QueryBuilder, QueryBuilderError, QueryOperation};

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

    if !builder.conditions.isEmpty() {
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

#[async_trait::async_trait]
impl DatabasePool for PostgresPool {
    async fn execute(&self, query: &str, params: Vec<Value>) -> Result<Vec<HashMap<String, Value>>, QueryBuilderError> {
        let client = self.pool.get().await.map_err(|e| QueryBuilderError::DatabaseError(e.to_string()))?;
        let stmt = client.prepare(query).await.map_err(|e| QueryBuilderError::DatabaseError(e.to_string()))?;
        let rows = client.query(&stmt, &params.iter().map(|v| v as &(dyn tokio_postgres::types::ToSql + Sync)).collect::<Vec<_>>())
            .await
            .map_err(|e| QueryBuilderError::DatabaseError(e.to_string()))?;

        Ok(rows.into_iter().map(|row| {
            row.columns().iter().map(|column| {
                (column.name().to_string(), row.get(column.name()))
            }).collect()
        }).collect())
    }
}