use sqlx::PgExecutor;
use uuid::Uuid;

#[tracing::instrument(name = "Save subscription token in the database", skip_all, err(Debug))]
pub async fn save_subscription_token(
    pg_executor: impl PgExecutor<'_>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
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
