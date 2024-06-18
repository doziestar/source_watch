use std::any::Any;
use async_trait::async_trait;
use mongodb::bson::Document;
use mongodb::{Cursor, error::Error};

/// Trait for Collection operations
/// The trait provides methods to interact with a collection in a database
/// The methods are asynchronous and return futures
#[async_trait]
pub trait CollectionOps: Any + Send + Sync {
    fn name(&self) -> &str;
    async fn find(&self, filter: Option<Document>) -> Result<Cursor<Document>, Error>;

}

/// Trait for Database interactions
/// The trait provides methods to interact with a database
/// The methods are asynchronous and return futures
/// The methods establish a connection, list collections, fetch documents, and query the database
/// The methods are implemented for different database types like SQL, Redis, and MongoDB
#[async_trait]
pub trait Database: Send + Sync {
    /// Establish a connection to the database
    /// The method takes a connection string as input
    /// It initializes the database connection and returns the database struct
    /// The method is asynchronous and returns a future
    /// The method is generic over the Database trait
    async fn establish_connection(connection_string: &str) -> Self where Self: Sized;

    /// List all collections in the database
    /// The method takes the database name as input
    /// It fetches all collections from the database
    async fn list_collections(&self, db_name: &str) -> Vec<String>;

    /// Fetch all documents from a collection in the database
    /// The method takes the database name and collection name as input
    /// It fetches all documents from the specified collection
    /// Returns a vector of strings representing the documents
    async fn fetch_all_documents(&self, db_name: &str, collection_name: &str) -> Vec<String>;

    /// Query the database with a specific query
    /// The method takes the database name, collection name, and query as input
    /// It queries the database with the specified query
    /// Returns a vector of strings representing the query results
    async fn query(&self, db_name: &str, collection_name: &str, query: &str) -> Vec<String>;

    /// Get a collection from the database
    /// The method takes the database name and collection name as input
    /// It fetches the collection from the database
    /// Returns the collection
    async fn get_collection(&self, db_name: &str, collection_name: &str) -> Box<dyn CollectionOps>;
}
