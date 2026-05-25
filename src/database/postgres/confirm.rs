use sqlx::PgExecutor;
use uuid::Uuid;

#[tracing::instrument(name = "Mark subscriber as confirmed", skip_all, err(Debug))]
pub async fn confirm_subscriber(
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

#[tracing::instrument(name = "Get subscriber_id from token", skip_all, err(Debug))]
pub async fn get_subscriber_id_from_token(
    pg_executor: impl PgExecutor<'_>,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT subscriber_id
        FROM subscription_tokens
        WHERE subscription_token = $1
        "#,
        subscription_token,
    )
    .fetch_optional(pg_executor)
    .await?;

    Ok(result.map(|r| r.subscriber_id))
}
