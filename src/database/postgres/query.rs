use chrono::Utc;
use secrecy::{ExposeSecret, Secret};
use sqlx::{postgres::PgQueryResult, PgExecutor};
use uuid::Uuid;

use crate::database::{
    base::{HeaderPairRecord, SavedHttpResponse},
    ConfirmedSubscriber, UserCredential,
};

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
        Utc::now(),
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
        VALUES ($1, $2);
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
    sqlx::query(r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1;"#)
        .bind(subscriber_id)
        .execute(pg_executor)
        .await
}

pub async fn pg_get_subscriber_id_from_token(
    pg_executor: impl PgExecutor<'_>,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT subscriber_id FROM subscription_tokens \
        WHERE subscription_token = $1;",
        subscription_token
    )
    .fetch_optional(pg_executor)
    .await?;
    Ok(result.map(|row| row.subscriber_id))
}

pub async fn pg_get_confirmed_subscribers(
    pg_executor: impl PgExecutor<'_>,
) -> Result<Vec<ConfirmedSubscriber>, sqlx::Error> {
    sqlx::query_as!(
        ConfirmedSubscriber,
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed';
        "#,
    )
    .fetch_all(pg_executor)
    .await
}

pub async fn pg_get_user_id(
    pg_executor: impl PgExecutor<'_>,
    username: &str,
    password_hash: Secret<String>,
) -> Result<Option<Uuid>, sqlx::Error> {
    let user_id = sqlx::query!(
        r#"
        SELECT user_id
        FROM users
        WHERE username = $1 AND password_hash = $2;
        "#,
        username,
        password_hash.expose_secret(),
    )
    .fetch_optional(pg_executor)
    .await?;
    Ok(user_id.map(|row| row.user_id))
}

pub async fn pg_get_user_credential(
    pg_executor: impl PgExecutor<'_>,
    username: &str,
) -> Result<Option<UserCredential>, sqlx::Error> {
    sqlx::query_as!(
        UserCredential,
        r#"
        SELECT user_id, password_hash
        FROM users
        WHERE username = $1
        "#,
        username
    )
    .fetch_optional(pg_executor)
    .await
}

pub async fn pg_get_username(
    pg_executor: impl PgExecutor<'_>,
    user_id: Uuid,
) -> Result<String, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT username
        FROM users
        WHERE user_id = $1;
        "#,
        user_id
    )
    .fetch_one(pg_executor)
    .await?;

    Ok(row.username)
}

pub async fn pg_change_password(
    pg_executor: impl PgExecutor<'_>,
    user_id: Uuid,
    password_hash: Secret<String>,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query!(
        "
        UPDATE users
        SET password_hash = $1
        WHERE user_id = $2
        ",
        password_hash.expose_secret(),
        user_id
    )
    .execute(pg_executor)
    .await
}

pub async fn pg_get_saved_response(
    pg_executor: impl PgExecutor<'_>,
    idempotency_key: &str,
    user_id: Uuid,
) -> Result<Option<SavedHttpResponse>, sqlx::Error> {
    sqlx::query_as!(
        SavedHttpResponse,
        r#"
    SELECT
        response_status_code,
        response_headers as "response_headers: Vec<HeaderPairRecord>",
        response_body
    FROM 
        idempotency
    WHERE 
        user_id = $1
        AND
        idempotency_key = $2
    "#,
        user_id,
        idempotency_key
    )
    .fetch_optional(pg_executor)
    .await
}
