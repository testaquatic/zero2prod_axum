use crate::{
    database::postgres::{self, PostgresDatabase},
    domain::new_subscriber::SubscribeFormData,
    service::error::ServiceError,
};

pub struct SubscribeService {
    postgres_database: PostgresDatabase,
}

impl SubscribeService {
    pub fn new(postgres_database: PostgresDatabase) -> Self {
        Self { postgres_database }
    }

    pub async fn subscribe(
        &self,
        subscriber_form_data: SubscribeFormData,
    ) -> Result<(), ServiceError> {
        let new_subscriber = subscriber_form_data
            .try_into()
            .map_err(ServiceError::ValidationError)?;

        postgres::subscription::insert_subscriber(&self.postgres_database, &new_subscriber).await?;

        Ok(())
    }
}
