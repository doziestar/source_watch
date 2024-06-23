use super::*;
use elasticsearch::{Elasticsearch, http::transport::Transport, SearchParts, CreateParts, UpdateParts, DeleteParts};
use elasticsearch::cat::CatIndicesParts;
use elasticsearch::http::request::JsonBody;
use serde_json::json;
use crate::db::query_builder::{Operator, QueryBuilder, QueryOperation, OrderDirection};
use crate::utils::errors::QueryBuilderError;

pub fn build_query(builder: &QueryBuilder) -> Result<String, QueryBuilderError> {
    let query = match builder.operation {
        QueryOperation::Select => build_search_query(builder),
        QueryOperation::Insert => build_create_query(builder),
        QueryOperation::Update => build_update_query(builder),
        QueryOperation::Delete => build_delete_query(builder),
        QueryOperation::Aggregate => build_aggregation_query(builder),
    }?;

    Ok(serde_json::to_string(&query).map_err(|e| QueryBuilderError::InvalidQuery(e.to_string()))?)
}

fn build_search_query(builder: &QueryBuilder) -> Result<serde_json::Value, QueryBuilderError> {
    let mut query = json!({
        "query": {
            "bool": {
                "must": []
            }
        }
    });

    for condition in &builder.conditions {
        let mut term = json!({});
        term[condition.field.as_str()] = json!({
            operator_to_elasticsearch(&condition.operator): condition.value
        });
        query["query"]["bool"]["must"].as_array_mut().unwrap().push(term);
    }

    if !builder.fields.is_empty() {
        query["_source"] = json!(builder.fields.iter().map(|f| f.as_str()).collect::<Vec<_>>());
    }

    if !builder.order_by.is_empty() {
        query["sort"] = json!(builder.order_by.iter().map(|o| {
            json!({
                o.field.as_str(): {
                    "order": if o.direction == OrderDirection::Asc { "asc" } else { "desc" }
                }
            })
        }).collect::<Vec<_>>());
    }

    if let Some(limit) = builder.limit {
        query["size"] = json!(limit);
    }

    if let Some(offset) = builder.offset {
        query["from"] = json!(offset);
    }

    Ok(query)
}

fn build_create_query(builder: &QueryBuilder) -> Result<serde_json::Value, QueryBuilderError> {
    if builder.fields.is_empty() || builder.values.is_empty() {
        return Err(QueryBuilderError::MissingField("fields or values".to_string()));
    }

    let doc: serde_json::Map<String, serde_json::Value> = builder.fields.iter()
        .zip(builder.values.iter())
        .map(|(f, v)| (f.as_str().to_string(), v.clone()))
        .collect();

    Ok(json!(doc))
}

fn build_update_query(builder: &QueryBuilder) -> Result<serde_json::Value, QueryBuilderError> {
    if builder.fields.is_empty() || builder.values.is_empty() {
        return Err(QueryBuilderError::MissingField("fields or values".to_string()));
    }

    let doc: serde_json::Map<String, serde_json::Value> = builder.fields.iter()
        .zip(builder.values.iter())
        .map(|(f, v)| (f.as_str().to_string(), v.clone()))
        .collect();

    Ok(json!({
        "doc": doc
    }))
}

fn build_delete_query(_builder: &QueryBuilder) -> Result<serde_json::Value, QueryBuilderError> {
    // Elasticsearch delete doesn't require a body, so we return an empty object
    Ok(json!({}))
}

fn build_aggregation_query(builder: &QueryBuilder) -> Result<serde_json::Value, QueryBuilderError> {
    let mut query = json!({
        "size": 0,
        "aggs": {}
    });

    for (i, field) in builder.fields.iter().enumerate() {
        let agg_name = format!("agg_{}", i);
        query["aggs"][&agg_name] = json!({
            "terms": {
                "field": field.as_str()
            }
        });
    }

    Ok(query)
}

fn operator_to_elasticsearch(operator: &Operator) -> &'static str {
    match operator {
        Operator::Eq => "term",
        Operator::Ne => "must_not",
        Operator::Gt => "gt",
        Operator::Lt => "lt",
        Operator::Gte => "gte",
        Operator::Lte => "lte",
        Operator::Like => "wildcard",
        Operator::In => "terms",
        Operator::NotIn => "must_not",
    }
}

pub struct ElasticsearchPool {
    client: Elasticsearch,
}

impl ElasticsearchPool {
    pub async fn new(connection_string: &str) -> Result<Self, DatabaseError> {
        let transport = elasticsearch::http::transport::Transport::single_node(connection_string)
            .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;
        let client = Elasticsearch::new(transport);
        Ok(ElasticsearchPool { client })
    }
}

#[async_trait::async_trait]
impl DatabasePool for ElasticsearchPool {
    async fn execute(&self, query: &str, _params: Vec<Value>) -> Result<Vec<HashMap<String, Value>>, DatabaseError> {
        let query: serde_json::Value = serde_json::from_str(query)
            .map_err(|e| DatabaseError::InvalidQuery(e.to_string()))?;

        let response = self.client
            .search(SearchParts::None)
            .body(query)
            .send()
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let response_body = response.json::<serde_json::Value>().await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let hits = response_body["hits"]["hits"]
            .as_array()
            .ok_or_else(|| DatabaseError::QueryError("Invalid response format".to_string()))?;

        Ok(hits
            .iter()
            .map(|hit| {
                hit["_source"]
                    .as_object()
                    .map(|obj| {
                        obj.iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect::<HashMap<String, Value>>()
                    })
                    .unwrap_or_default()
            })
            .collect())
    }
}

#[async_trait::async_trait]
impl Database for ElasticsearchPool {
    async fn connect(connection_string: &str) -> Result<Self, DatabaseError> {
        Self::new(connection_string).await
    }

    async fn disconnect(&self) -> Result<(), DatabaseError> {
        // Elasticsearch client doesn't require explicit disconnection
        Ok(())
    }

    async fn execute_query(&self, query: &str) -> Result<Vec<Value>, DatabaseError> {
        let query: serde_json::Value = serde_json::from_str(query)
            .map_err(|e| DatabaseError::InvalidQuery(e.to_string()))?;

        let response = self.client
            .search(SearchParts::None)
            .body(query)
            .send()
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let response_body = response.json::<serde_json::Value>().await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(vec![response_body])
    }

    async fn list_databases(&self) -> Result<Vec<String>, DatabaseError> {
        // Elasticsearch doesn't have a concept of databases, return indices instead
        let response = self.client
            .cat()
            .indices(CatIndicesParts::None)
            .format("json")
            .send()
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let indices: Vec<serde_json::Value> = response.json().await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(indices
            .into_iter()
            .filter_map(|index| index["index"].as_str().map(String::from))
            .collect())
    }

    async fn list_collections(&self, index: &str) -> Result<Vec<String>, DatabaseError> {
        // Elasticsearch doesn't have collections, return mappings instead
        let response = self.client
            .indices()
            .get_mapping(elasticsearch::indices::IndicesGetMappingParts::Index(&[index]))
            .send()
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let mappings: serde_json::Value = response.json().await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(mappings[index]["mappings"]["properties"]
            .as_object()
            .map(|obj| obj.keys().cloned().collect())
            .unwrap_or_default())
    }

    async fn get_schema(&self, index: &str, _collection: &str) -> Result<Value, DatabaseError> {
        let response = self.client
            .indices()
            .get_mapping(elasticsearch::indices::IndicesGetMappingParts::Index(&[index]))
            .send()
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let mappings: serde_json::Value = response.json().await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(mappings[index]["mappings"].clone())
    }
}