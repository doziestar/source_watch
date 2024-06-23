use super::*;
use mongodb::{bson::{doc, Document, Bson}, bson, options::FindOptions};
use serde_json::json;
use crate::db::query_builder::{Condition, Operator, OrderDirection, QueryBuilder, QueryOperation};
use crate::utils::errors::{QueryBuilderError, DatabaseError};

pub fn build_query(builder: &QueryBuilder) -> Result<String, QueryBuilderError> {
    let query = match builder.operation {
        QueryOperation::Select => build_find_query(builder),
        QueryOperation::Insert => build_insert_query(builder),
        QueryOperation::Update => build_update_query(builder),
        QueryOperation::Delete => build_delete_query(builder),
        QueryOperation::Aggregate => build_aggregate_query(builder),
    }?;

    Ok(serde_json::to_string(&query).map_err(|e| QueryBuilderError::InvalidQuery(e.to_string()))?)
}

fn build_find_query(builder: &QueryBuilder) -> Result<Document, QueryBuilderError> {
    let mut query = doc! {};
    let mut options = FindOptions::default();

    if !builder.conditions.is_empty() {
        query = build_filter(&builder.conditions)?;
    }

    if !builder.fields.is_empty() {
        let projection: Document = builder.fields.iter()
            .map(|f| (f.as_str().to_string(), Bson::Int32(1)))
            .collect();
        options.projection = Some(projection);
    }

    if !builder.order_by.is_empty() {
        let sort: Document = builder.order_by.iter()
            .map(|o| {
                (o.field.as_str().to_string(),
                 if o.direction == OrderDirection::Asc { Bson::Int32(1) } else { Bson::Int32(-1) })
            })
            .collect();
        options.sort = Some(sort);
    }

    if let Some(limit) = builder.limit {
        options.limit = Some(limit as i64);
    }

    if let Some(offset) = builder.offset {
        options.skip = Some(offset as u64);
    }

    Ok(doc! {
        "find": builder.table.as_str(),
        "filter": query,
        "options": bson::to_bson(&options).map_err(|e| QueryBuilderError::InvalidQuery(e.to_string()))?
    })
}


fn build_insert_query(builder: &QueryBuilder) -> Result<Document, QueryBuilderError> {
    if builder.fields.is_empty() || builder.values.is_empty() {
        return Err(QueryBuilderError::MissingField("fields or values".to_string()));
    }

    let doc: Document = builder.fields.iter().zip(builder.values.iter())
        .map(|(f, v)| (f.as_str().to_string(), bson::to_bson(v).unwrap()))
        .collect();

    Ok(doc! {
        "insert": builder.table.as_str(),
        "documents": [doc]
    })
}

fn build_update_query(builder: &QueryBuilder) -> Result<Document, QueryBuilderError> {
    if builder.fields.is_empty() || builder.values.is_empty() {
        return Err(QueryBuilderError::MissingField("fields or values".to_string()));
    }

    let filter = if !builder.conditions.is_empty() {
        build_filter(&builder.conditions)?
    } else {
        doc! {}
    };

    let update = doc! {
        "$set": builder.fields.iter().zip(builder.values.iter())
            .map(|(f, v)| (f.as_str().to_string(), bson::to_bson(v).unwrap()))
            .collect::<Document>()
    };

    Ok(doc! {
        "update": builder.table.as_str(),
        "updates": [{
            "q": filter,
            "u": update,
            "multi": true
        }]
    })
}

fn build_delete_query(builder: &QueryBuilder) -> Result<Document, QueryBuilderError> {
    let filter = if !builder.conditions.is_empty() {
        build_filter(&builder.conditions)?
    } else {
        doc! {}
    };

    Ok(doc! {
        "delete": builder.table.as_str(),
        "deletes": [{
            "q": filter,
            "limit": 0
        }]
    })
}

fn build_aggregate_query(builder: &QueryBuilder) -> Result<Document, QueryBuilderError> {
    let mut pipeline = Vec::new();

    if !builder.conditions.is_empty() {
        pipeline.push(doc! {
            "$match": build_filter(&builder.conditions)?
        });
    }

    if !builder.fields.is_empty() {
        // let project: Document = builder.fields.iter().map(|f| (f.as_str().to_string(), 1)).collect();
        pipeline.push(doc! {
            // "$project": builder.fields.iter().map(|f| (f.as_str().to_string(), 1)).collect::<Document>()
        });
    }

    if !builder.order_by.is_empty() {
        println!("Order by not supported in MongoDB aggregate")
    }

    if let Some(offset) = builder.offset {
        pipeline.push(doc! {
            "$skip": offset as i64
        });
    }

    if let Some(limit) = builder.limit {
        pipeline.push(doc! {
            "$limit": limit as i64
        });
    }

    Ok(doc! {
        "aggregate": builder.table.as_str(),
        "pipeline": pipeline
    })
}

fn build_filter(conditions: &[Condition]) -> Result<Document, QueryBuilderError> {
    let mut filter = Document::new();
    for condition in conditions {
        let value = bson::to_bson(&condition.value).map_err(|e| QueryBuilderError::InvalidQuery(e.to_string()))?;
        match condition.operator {
            Operator::Eq => { filter.insert(condition.field.as_str(), value); }
            Operator::Ne => { filter.insert(condition.field.as_str(), doc! { "$ne": value }); }
            Operator::Gt => { filter.insert(condition.field.as_str(), doc! { "$gt": value }); }
            Operator::Lt => { filter.insert(condition.field.as_str(), doc! { "$lt": value }); }
            Operator::Gte => { filter.insert(condition.field.as_str(), doc! { "$gte": value }); }
            Operator::Lte => { filter.insert(condition.field.as_str(), doc! { "$lte": value }); }
            Operator::Like => { filter.insert(condition.field.as_str(), doc! { "$regex": value, "$options": "i" }); }
            Operator::In => { filter.insert(condition.field.as_str(), doc! { "$in": value }); }
            Operator::NotIn => { filter.insert(condition.field.as_str(), doc! { "$nin": value }); }
        }
    }
    Ok(filter)
}

pub struct MongoPool {
    client: mongodb::Client,
    db_name: String,
}

impl MongoPool {
    pub async fn new(connection_string: &str) -> Result<Self, DatabaseError> {
        let client = mongodb::Client::with_uri_str(connection_string).await
            .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;
        let db_name = "test".to_string();
        Ok(MongoPool { client, db_name })
    }
}

#[async_trait::async_trait]
impl DatabasePool for MongoPool {
    async fn execute(&self, query: &str, _params: Vec<Value>) -> Result<Vec<HashMap<String, Value>>, DatabaseError> {
        let db = self.client.database(&self.db_name);
        let command: Document = serde_json::from_str(query).map_err(|e| DatabaseError::InvalidQuery(e.to_string()))?;

        let result = db.run_command(command, None).await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let result_vec = match result.get("cursor") {
            Some(cursor) => {
                cursor.as_document()
                    .and_then(|doc| doc.get("firstBatch"))
                    .and_then(|batch| batch.as_array())
                    .map(|arr| arr.to_vec())
                    .unwrap_or_default()
            },
            None => vec![Bson::Document(result)],
        };

        Ok(result_vec.into_iter().map(|doc| {
            doc.as_document().unwrap().iter().map(|(k, v)| {
                (k.to_string(), bson_to_json(v))
            }).collect()
        }).collect())
    }
}

fn bson_to_json(bson: &Bson) -> Value {
    match bson {
        Bson::Double(v) => json!(v),
        Bson::String(v) => json!(v),
        Bson::Array(v) => json!(v.iter().map(bson_to_json).collect::<Vec<_>>()),
        Bson::Document(v) => {
            let map: serde_json::Map<String, Value> = v.iter().map(|(k, v)| (k.to_string(), bson_to_json(v))).collect();
            Value::Object(map)
        },
        Bson::Boolean(v) => json!(v),
        Bson::Null => json!(null),
        Bson::Int32(v) => json!(v),
        Bson::Int64(v) => json!(v),
        Bson::DateTime(v) => json!(v.to_string()),
        _ => json!(bson.to_string()),
    }
}