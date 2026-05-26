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

pub async fn get_user_id_from_credentials(
    pg_executor: impl PgExecutor<'_>,
    username: &str,
    password_hash: &str,
) -> Result<Option<uuid::Uuid>, sqlx::Error> {
    sqlx::query!(
        r#"
        SELECT user_id
        FROM users
        WHERE username = $1 and password_hash = $2
        "#,
        username,
        password_hash,
    )
    .fetch_optional(pg_executor)
    .await
    .map(|row| row.map(|user| user.user_id))
}
