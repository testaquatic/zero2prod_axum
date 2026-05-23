use crate::{
    database::PostgresDatabase, domain::subscriber::SubscribeData, service::error::ServiceError,
};

pub struct SubscribeService {
    postgres_database: PostgresDatabase,
}

impl SubscribeService {
    pub fn new(postgres_database: PostgresDatabase) -> Self {
        Self { postgres_database }
    }

    pub async fn subscribe(&self, subscriber_data: &SubscribeData) -> Result<(), ServiceError> {
        self.postgres_database
            .insert_subscriber(&subscriber_data.email, &subscriber_data.name)
            .await?;

        Ok(())
    }
}
