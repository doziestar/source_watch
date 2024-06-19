use crate::db::db::{CollectionOps, Database};
use async_trait::async_trait;
use mockall::mock;

mock! {
    pub RedisDatabase {}

    #[async_trait]
    impl Database for RedisDatabase {
        async fn establish_connection(redis_url: &str) -> Self {
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
async fn test_redis_establish_connection() {
    let mock = MockRedisDatabase::establish_connection("redis://localhost:6379").await;
    assert!(mock.is_some());
}
