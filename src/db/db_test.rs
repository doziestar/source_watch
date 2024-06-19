#[cfg(test)]
mod tests {
    use crate::db::db::CollectionOps;
    use async_trait::async_trait;
    use mockall::mock;
    use mongodb::bson::Document;
    use mongodb::{error::Error, Cursor};

    mock! {
        pub CollectionType {}

        #[async_trait]
        impl CollectionOps for CollectionType {
            fn name(&self) -> &str {
                ""
            }

            async fn find(&self, _filter: Option<Document>) -> Result<Cursor<Document>, Error> {
                Ok(Cursor::new(vec![]))
            }
        }
    }
}
