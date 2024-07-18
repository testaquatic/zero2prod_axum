use chrono::Utc;
use sqlx::{postgres::PgQueryResult, PgExecutor};
use uuid::Uuid;

pub async fn pg_save_subscriber(
    pg_executor: impl PgExecutor<'_>,
    email: &str,
    name: &str,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4);
        "#,
        Uuid::new_v4(),
        email,
        name,
        Utc::now()
    )
    .execute(pg_executor)
    .await
}
