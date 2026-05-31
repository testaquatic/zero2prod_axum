use chrono::Utc;
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::domain::new_subscriber::NewSubscriber;

/// 구독자 데이터를 데이터베이스에 저장한다.
/// 구독자의 `Uuid`를 반환한다.
#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip_all,
    err(Debug)
)]
pub async fn save_subscriber(
    pg_executor: impl PgExecutor<'_>,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
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

#[tracing::instrument(name = "Mark subscriber as confirmed", skip_all, err(Debug))]
pub async fn update_subscriber_confirmed(
    pg_executor: impl PgExecutor<'_>,
    subscriber_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE subscriptions
        SET status = 'confirmed'
        WHERE id = $1
        "#,
        subscriber_id,
    )
    .execute(pg_executor)
    .await?;

    Ok(())
}
