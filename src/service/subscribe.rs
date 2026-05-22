use crate::{database::PostgresDatabase, service::error::ServiceError};

pub struct SubscribeService {
    postgres_database: PostgresDatabase,
}

impl SubscribeService {
    pub fn new(postgres_database: PostgresDatabase) -> Self {
        Self { postgres_database }
    }

    pub async fn subscribe(&self, email: &str, name: &str) -> Result<(), ServiceError> {
        self.postgres_database.subscribe(email, name).await?;

        Ok(())
    }
}
