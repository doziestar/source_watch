use std::sync::Arc;
use async_trait::async_trait;
use mongodb::bson::Document;
use mongodb::{Client, options::ClientOptions, Collection, Cursor, error::Error};
use futures::stream::StreamExt;
use crate::db::db::{CollectionOps, Database};

/// MongoDB collection type
pub struct MongoCollectionType {
    collection: Collection<Document>,
}

#[async_trait]
impl CollectionOps for MongoCollectionType {
    fn name(&self) -> &str {
        self.collection.name()
    }

    async fn find(&self, filter: Option<Document>) -> Result<Cursor<Document>, Error> {
        self.collection.find(filter, None).await
    }
}

#[derive(Debug)]
pub struct MongoDatabase {
    client: Client,
}

#[async_trait]
impl Database for MongoDatabase {
    async fn establish_connection(mongo_url: &str) -> Self {
        let client_options = ClientOptions::parse(mongo_url).await.expect("Failed to parse options");
        let client = Client::with_options(client_options).expect("Failed to initialize client");
        MongoDatabase { client }
    }

    async fn list_collections(&self, db_name: &str) -> Vec<String> {
        let database = self.client.database(db_name);
        database.list_collection_names(None).await.expect("Failed to list collections")
    }

    async fn fetch_all_documents(&self, db_name: &str, collection_name: &str) -> Vec<String> {
        let collection = self.get_collection_as_mongo(db_name, collection_name).await;
        let mut cursor = collection.find(None).await.expect("Failed to fetch documents");

        let mut documents = Vec::new();
        while let Some(result) = cursor.next().await {
            match result {
                Ok(document) => documents.push(document.to_string()),
                Err(e) => eprintln!("Error retrieving document: {}", e),
            }
        }

        documents
    }

    async fn query(&self, db_name: &str, collection_name: &str, query: &str) -> Vec<String> {
        let collection = self.get_collection_as_mongo(db_name, collection_name).await;

        let filter: Document = serde_json::from_str(query).expect("Failed to parse query");
        let mut cursor = collection.find(Some(filter)).await.expect("Failed to execute query");

        let mut documents = Vec::new();
        while let Some(result) = cursor.next().await {
            match result {
                Ok(document) => documents.push(document.to_string()),
                Err(e) => eprintln!("Error retrieving document: {}", e),
            }
        }

        documents
    }

    async fn get_collection(&self, db_name: &str, collection_name: &str) -> Box<dyn CollectionOps> {
        Box::new(self.get_collection_as_mongo(db_name, collection_name).await)
    }
}

impl MongoDatabase {
    async fn get_collection_as_mongo(&self, db_name: &str, collection_name: &str) -> MongoCollectionType {
        let database = self.client.database(db_name);
        let collection = database.collection::<Document>(collection_name);
        MongoCollectionType { collection }
    }
}
