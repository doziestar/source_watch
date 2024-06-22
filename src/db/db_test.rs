#[cfg(test)]
mod tests {
    use super::*;
    use mockall::automock;
    use mockall::predicate::*;
    use serde_json::json;
    use std::collections::HashMap;
    use async_trait::async_trait;

    #[automock]
    #[async_trait]
    pub trait DatabasePool: Send + Sync {
        async fn execute(&self, query: &str, params: Vec<Value>) -> Result<Vec<HashMap<String, Value>>, query_builder::QueryBuilderError>;
    }

    #[tokio::test]
    async fn test_execute() {
        let mut mock = MockDatabasePool::new();
        mock.expect_execute()
            .with(eq("SELECT * FROM users WHERE id = $1"), eq(vec![json!(1)]))
            .times(1)
            .returning(|_, _| {
                Ok(vec![HashMap::new()])
            });

        let result = mock.execute("SELECT * FROM users WHERE id = $1", vec![json!(1)]).await;
        assert!(result.is_ok());
    }
}
