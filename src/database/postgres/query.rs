use chrono::Utc;
use sqlx::{postgres::PgQueryResult, PgExecutor};
use uuid::Uuid;

pub async fn pg_insert_subscriber(
    pg_executor: impl PgExecutor<'_>,
    email: &str,
    name: &str,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation');
        "#,
        subscriber_id,
        email,
        name,
        Utc::now()
    )
    .execute(pg_executor)
    .await?;

    Ok(subscriber_id)
}

pub async fn pg_store_token(
    pg_executor: impl PgExecutor<'_>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2)
        "#,
        subscription_token,
        subscriber_id
    )
    .execute(pg_executor)
    .await
}

// 두번 요청이 들어오더라도 오류를 반환하지 않는다.
pub async fn pg_confirm_subscriber(
    pg_executor: impl PgExecutor<'_>,
    subscriber_id: Uuid,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id
    )
    .execute(pg_executor)
    .await
}

pub async fn pg_get_subscriber_id_from_token(
    pg_executor: impl PgExecutor<'_>,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT subscriber_id FROM subscription_tokens \
        WHERE subscription_token = $1",
        subscription_token,
    )
    .fetch_optional(pg_executor)
    .await?;
    Ok(result.map(|r| r.subscriber_id))
}
