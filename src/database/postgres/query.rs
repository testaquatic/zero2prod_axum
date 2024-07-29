use chrono::Utc;
use secrecy::{ExposeSecret, Secret};
use sqlx::{postgres::PgQueryResult, PgExecutor, Row};
use uuid::Uuid;

use crate::database::ConfirmedSubscriber;

pub async fn pg_insert_subscriber(
    pg_executor: impl PgExecutor<'_>,
    email: &str,
    name: &str,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation');
        "#,
    )
    .bind(subscriber_id)
    .bind(email)
    .bind(name)
    .bind(Utc::now())
    .execute(pg_executor)
    .await?;

    Ok(subscriber_id)
}

pub async fn pg_store_token(
    pg_executor: impl PgExecutor<'_>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2);
        "#,
    )
    .bind(subscription_token)
    .bind(subscriber_id)
    .execute(pg_executor)
    .await
}

// 두번 요청이 들어오더라도 오류를 반환하지 않는다.
pub async fn pg_confirm_subscriber(
    pg_executor: impl PgExecutor<'_>,
    subscriber_id: Uuid,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query(r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1;"#)
        .bind(subscriber_id)
        .execute(pg_executor)
        .await
}

pub async fn pg_get_subscriber_id_from_token(
    pg_executor: impl PgExecutor<'_>,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query(
        "SELECT subscriber_id FROM subscription_tokens \
        WHERE subscription_token = $1;",
    )
    .bind(subscription_token)
    .fetch_optional(pg_executor)
    .await?;
    match result {
        Some(row) => Ok(row.try_get("subscriber_id")?),
        None => Ok(None),
    }
}

pub async fn pg_get_confirmed_subscribers(
    pg_executor: impl PgExecutor<'_>,
) -> Result<Vec<ConfirmedSubscriber>, sqlx::Error> {
    let subscribers: Vec<ConfirmedSubscriber> = sqlx::query_as(
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed';
        "#,
    )
    .fetch_all(pg_executor)
    .await?;
    Ok(subscribers)
}

pub async fn pg_validate_credentials(
    pg_executor: impl PgExecutor<'_>,
    username: &str,
    password: Secret<String>,
) -> Result<Option<Uuid>, sqlx::Error> {
    let user_id = sqlx::query(
        r#"
        SELECT user_id
        FROM users
        WHERE username = $1 AND password = $2;
        "#,
    )
    .bind(username)
    .bind(password.expose_secret())
    .fetch_optional(pg_executor)
    .await?;
    match user_id {
        Some(user_id) => Ok(user_id.try_get("user_id")?),
        None => Ok(None),
    }
}
