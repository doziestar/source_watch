use mongodb::{Client, options::ClientOptions, Collection};
use mongodb::bson::Document;
use std::env;
use tokio::runtime::Runtime;
use async_trait::async_trait;
use futures::stream::StreamExt;

#[async_trait]
pub trait Database {
    async fn establish_connection(mongo_url: &str) -> Self;
    async fn get_collection(&self, db_name: &str, collection_name: &str) -> Collection<Document>;
    async fn list_collections(&self, db_name: &str) -> Vec<String>;
    async fn fetch_all_documents(&self, db_name: &str, collection_name: &str) -> Vec<Document>;
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

    async fn get_collection(&self, db_name: &str, collection_name: &str) -> Collection<Document> {
        let database = self.client.database(db_name);
        database.collection(collection_name)
    }

    async fn list_collections(&self, db_name: &str) -> Vec<String> {
        let databases = self.client.list_database_names(None, None).await.expect("Failed to list databases");
        println!("Listing databases: {:?}", databases);
        let database = self.client.database(db_name);
        let collections = database.list_collection_names(None).await.expect("Failed to list collections");
        collections
    }

    async fn fetch_all_documents(&self, db_name: &str, collection_name: &str) -> Vec<Document> {
        let collection = self.get_collection(db_name, collection_name).await;
        let mut cursor = collection.find(None, None).await.expect("Failed to fetch documents");

        let mut documents = Vec::new();
        while let Some(result) = cursor.next().await {
            match result {
                Ok(document) => documents.push(document),
                Err(e) => eprintln!("Error retrieving document: {}", e),
            }
        }

        documents
    }
}

pub fn get_mongo_document() {
    let mongo_url = env::var("MONGO_URL").unwrap_or_else(|_| "your_mongo_url".to_string());
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let mongo_db = MongoDatabase::establish_connection(&mongo_url).await;

        println!("MongoDatabase established: {:?}", mongo_db);

        // Specify the database name
        let db_name = "HubHub_DB";

        // List and print all collections
        let collections = mongo_db.list_collections(db_name).await;
        println!("Collections in the database: {:?}", collections);

        // Fetch and print all documents from a specific collection
        let collection_name = "user";
        let documents = mongo_db.fetch_all_documents(db_name, collection_name).await;

        println!("Documents in the collection {}: {:?}", collection_name, documents);
    });
}