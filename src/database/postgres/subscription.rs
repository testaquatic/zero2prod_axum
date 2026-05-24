use chrono::Utc;

use crate::{
    database::{error::DatabaseError, postgres::PostgresDatabase},
    domain::new_subscriber::NewSubscriber,
};

#[tracing::instrument(name = "Saving new subscriber details in the database", skip_all)]
pub async fn insert_subscriber(
    postgres_database: &PostgresDatabase,
    new_subscriber: &NewSubscriber,
) -> Result<(), DatabaseError> {
    sqlx::query!(
        "INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4);",
        uuid::Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now(),
    )
    .execute(postgres_database.pool())
    .await?;

    Ok(())
}
