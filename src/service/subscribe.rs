use crate::{
    database::postgres::{self, PostgresDatabase},
    domain::subscriber::{NewSubscriber, SubscriberName},
    handler::subscriptions::SubscribeData,
    service::error::ServiceError,
};

pub struct SubscribeService {
    postgres_database: PostgresDatabase,
}

impl SubscribeService {
    pub fn new(postgres_database: PostgresDatabase) -> Self {
        Self { postgres_database }
    }

    pub async fn subscribe(&self, subscriber_data: SubscribeData) -> Result<(), ServiceError> {
        let new_subscriber = NewSubscriber {
            email: subscriber_data.email,
            name: SubscriberName::parse(subscriber_data.name)
                .map_err(ServiceError::ValidationError)?,
        };

        postgres::subscription::insert_subscriber(&self.postgres_database, &new_subscriber).await?;

        Ok(())
    }
}
