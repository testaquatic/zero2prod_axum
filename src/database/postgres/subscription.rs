use chrono::Utc;
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::{database::error::DatabaseError, domain::new_subscriber::NewSubscriber};

/// 구독자 데이터를 데이터베이스에 저장한다.
/// 구독자의 `Uuid`를 반환한다.
#[tracing::instrument(name = "Saving new subscriber details in the database", skip_all)]
pub async fn insert_subscriber(
    pg_executor: impl PgExecutor<'_>,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, DatabaseError> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status) 
        VALUES ($1, $2, $3, $4, 'pending_confirmation');
        "#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now(),
    )
    .execute(pg_executor)
    .await?;

    Ok(subscriber_id)
}

#[tracing::instrument(name = "Store subscription token in the database", skip_all)]
pub async fn store_token(
    pg_executor: impl PgExecutor<'_>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), DatabaseError> {
    sqlx::query!(
        r#"
        INSERT INTO subscription_tokens (subscription_token, subscriber_id) 
        VALUES ($1, $2);
        "#,
        subscription_token,
        subscriber_id,
    )
    .execute(pg_executor)
    .await?;

    Ok(())
}
