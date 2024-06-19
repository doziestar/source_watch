use crate::db::db::{CollectionOps, Database};
use async_trait::async_trait;
use mockall::predicate::*;
use mockall::*;

mock! {
    pub MongoDatabase {}

    #[async_trait]
    impl Database for MongoDatabase {
        async fn establish_connection(mongo_url: &str) -> Self {
            Self
        }

        async fn list_collections(&self, db_name: &str) -> Vec<String> {
            vec![]
        }

        async fn fetch_all_documents(&self, db_name: &str, collection_name: &str) -> Vec<String> {
            vec![]
        }

        async fn query(&self, db_name: &str, collection_name: &str, query: &str) -> Vec<String> {
            vec![]
        }

        async fn get_collection(&self, db_name: &str, collection_name: &str) -> Box<dyn CollectionOps> {
            Box::new(MockCollectionType)
        }
    }
}

#[tokio::test]
async fn test_mongo_establish_connection() {
    let mock = MockMongoDatabase::establish_connection("mongodb://localhost:27017").await;
    assert!(mock.is_some());
}

#[tokio::test]
async fn test_list_collections() {
    let mut mock = MockMongoDatabase::default();
    mock.expect_list_collections()
        .with(predicate::eq("test_db"))
        .times(1)
        .returning(|_| vec!["collection1".to_string(), "collection2".to_string()]);

    let collections = mock.list_collections("test_db").await;
    assert_eq!(collections, vec!["collection1", "collection2"]);
}

#[tokio::test]
async fn test_fetch_all_documents() {
    let mut mock = MockMongoDatabase::default();
    mock.expect_fetch_all_documents()
        .with(predicate::eq("test_db"), predicate::eq("test_collection"))
        .times(1)
        .returning(|_, _| vec!["doc1".to_string(), "doc2".to_string()]);

    let documents = mock.fetch_all_documents("test_db", "test_collection").await;
    assert_eq!(documents, vec!["doc1", "doc2"]);
}

#[tokio::test]
async fn test_query() {
    let mut mock = MockMongoDatabase::default();
    mock.expect_query()
        .with(
            predicate::eq("test_db"),
            predicate::eq("test_collection"),
            predicate::eq("{\"key\":\"value\"}"),
        )
        .times(1)
        .returning(|_, _, _| vec!["doc1".to_string()]);

    let results = mock
        .query("test_db", "test_collection", "{\"key\":\"value\"}")
        .await;
    assert_eq!(results, vec!["doc1"]);
}

#[tokio::test]
async fn test_get_collection() {
    let mock = MockMongoDatabase::default();
    let collection = mock.get_collection("test_db", "test_collection").await;
    assert_eq!(collection.name(), "test_collection");
}
