use crate::db::db::{CollectionOps, Database};
use async_trait::async_trait;
use mongodb::bson::Document;
use mongodb::error::Error;
use mongodb::Cursor;
use redis::AsyncCommands;

#[derive(Debug)]
pub struct RedisDatabase {
    client: redis::Client,
}

pub struct RedisCollectionType {
    name: String,
}

#[async_trait]
impl CollectionOps for RedisCollectionType {
    fn name(&self) -> &str {
        &self.name
    }

    async fn find(&self, _filter: Option<Document>) -> Result<Cursor<Document>, Error> {
        unimplemented!() // Redis doesn't use MongoDB's find method, so this is a placeholder.
    }
}

#[async_trait]
impl Database for RedisDatabase {
    async fn establish_connection(redis_url: &str) -> Self {
        let client = redis::Client::open(redis_url).expect("Failed to create Redis client");
        RedisDatabase { client }
    }

    async fn list_collections(&self, _db_name: &str) -> Vec<String> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to connect to Redis");
        let keys: Vec<String> = con.keys("*").await.expect("Failed to list keys");
        keys
    }

    async fn fetch_all_documents(&self, _db_name: &str, collection_name: &str) -> Vec<String> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to connect to Redis");
        let value: String = con.get(collection_name).await.expect("Failed to get value");
        vec![value]
    }

    async fn query(&self, _db_name: &str, collection_name: &str, query: &str) -> Vec<String> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to connect to Redis");
        let value: String = con.get(collection_name).await.expect("Failed to get value");
        if value.contains(query) {
            vec![value]
        } else {
            vec![]
        }
    }

    async fn get_collection(
        &self,
        _db_name: &str,
        collection_name: &str,
    ) -> Box<dyn CollectionOps> {
        Box::new(RedisCollectionType {
            name: collection_name.to_string(),
        })
    }
}
