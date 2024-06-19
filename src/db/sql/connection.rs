use crate::db::db::{CollectionOps, Database};
use async_trait::async_trait;
use mongodb::bson::Document;
use mongodb::error::Error;
use mongodb::Cursor;
use serde_json::Value;
use sqlx::{Column, PgPool, Row, TypeInfo};

pub struct SqlCollectionType {
    name: String,
}

#[async_trait]
impl CollectionOps for SqlCollectionType {
    fn name(&self) -> &str {
        &self.name
    }

    async fn find(&self, _filter: Option<Document>) -> Result<Cursor<Document>, Error> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct SqlDatabase {
    pool: PgPool,
}

#[async_trait]
impl Database for SqlDatabase {
    async fn establish_connection(pg_url: &str) -> Self {
        let pool = PgPool::connect(pg_url)
            .await
            .expect("Failed to create Postgres pool");
        SqlDatabase { pool }
    }

    async fn list_collections(&self, db_name: &str) -> Vec<String> {
        let query = format!(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = '{}'",
            db_name
        );
        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .expect("Failed to list tables");
        rows.into_iter().map(|row| row.get("table_name")).collect()
    }

    async fn fetch_all_documents(&self, _db_name: &str, collection_name: &str) -> Vec<String> {
        let query = format!("SELECT * FROM {}", collection_name);
        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .expect("Failed to fetch documents");

        rows.into_iter()
            .map(|row| row_to_json_string(&row))
            .collect()
    }

    async fn query(&self, _db_name: &str, collection_name: &str, query: &str) -> Vec<String> {
        let query = format!("SELECT * FROM {} WHERE {}", collection_name, query);
        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .expect("Failed to query database");

        rows.into_iter()
            .map(|row| row_to_json_string(&row))
            .collect()
    }

    async fn get_collection(
        &self,
        _db_name: &str,
        collection_name: &str,
    ) -> Box<dyn CollectionOps> {
        Box::new(SqlCollectionType {
            name: collection_name.to_string(),
        })
    }
}

fn row_to_json_string(row: &sqlx::postgres::PgRow) -> String {
    let json_value: Value = row_to_json(row);
    serde_json::to_string(&json_value).expect("Failed to convert row to JSON")
}

fn row_to_json(row: &sqlx::postgres::PgRow) -> Value {
    let mut map = serde_json::Map::new();
    for column in row.columns() {
        let value = match column.type_info().name() {
            "BOOL" => row.try_get::<bool, _>(column.name()).map(Value::Bool),
            "INT4" | "INT8" => row
                .try_get::<i64, _>(column.name())
                .map(|v| Value::Number(v.into())),
            "FLOAT4" | "FLOAT8" => row
                .try_get::<f64, _>(column.name())
                .map(|v| Value::Number(serde_json::Number::from_f64(v).unwrap())),
            "TEXT" | "VARCHAR" => row.try_get::<String, _>(column.name()).map(Value::String),
            _ => Ok(Value::Null),
        };
        map.insert(column.name().to_string(), value.unwrap_or(Value::Null));
    }
    Value::Object(map)
}
