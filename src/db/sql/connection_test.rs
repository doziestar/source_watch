use crate::db::db::{CollectionOps, Database};
use async_trait::async_trait;
use mockall::mock;

mock! {
    pub SqlDatabase {}

    #[async_trait]
    impl Database for SqlDatabase {
        async fn establish_connection(pg_url: &str) -> Self {
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
async fn test_sql_establish_connection() {
    let mock = MockSqlDatabase::establish_connection("postgres://localhost:5432").await;
    assert!(mock.is_some());
}
