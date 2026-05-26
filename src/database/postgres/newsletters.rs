use sqlx::PgExecutor;

pub struct ConfirmedSubscriber {
    pub email: String,
}

#[tracing::instrument(name = "Get confirmed subscribers", skip_all, err(Debug))]
pub async fn get_confirmed_subscribers(
    pg_executor: impl PgExecutor<'_>,
) -> Result<Vec<ConfirmedSubscriber>, sqlx::Error> {
    sqlx::query_as!(
        ConfirmedSubscriber,
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(pg_executor)
    .await
}
