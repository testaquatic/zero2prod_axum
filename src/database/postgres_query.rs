// 쿼리와 로직을 분리한다.

use secrecy::{ExposeSecret, Secret};
use sqlx::{postgres::PgQueryResult, PgExecutor};
use uuid::Uuid;

use crate::database::{
    types::{HeaderPairRecord, SavedHttpResponse},
    UserCredential,
};

use super::types::NewsletterIssue;

pub async fn pg_insert_subscriber(
    pg_executor: impl PgExecutor<'_>,
    email: &str,
    name: &str,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        -- 버그인지 `chrono::offset::Utc::now()`를 사용하면 타입 불일치 오류가 발생한다.
        -- SQL의 내장 함수인 `now()`로 대체했다.
        VALUES ($1, $2, $3, now(), 'pending_confirmation');
        "#,
        subscriber_id,
        email,
        name
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

/*
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
*/

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
        response_status_code as "response_status_code!",
        response_headers as "response_headers!: Vec<HeaderPairRecord>",
        response_body as "response_body!"
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

pub async fn pg_save_response(
    pg_executor: impl PgExecutor<'_>,
    user_id: Uuid,
    idempotency_key: &str,
    status_code: i16,
    headers: &[HeaderPairRecord],
    body: &[u8],
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query_unchecked!(
        r#"
        UPDATE idempotency
        SET 
            response_status_code = $3,
            response_headers = $4,
            response_body = $5
        WHERE 
            user_id = $1
            AND
            idempotency_key = $2;
        "#,
        user_id,
        idempotency_key,
        status_code,
        headers,
        body
    )
    .execute(pg_executor)
    .await
}

pub async fn pg_try_saving_idempotency_key(
    pg_executor: impl PgExecutor<'_>,
    idempotency_key: &str,
    user_id: Uuid,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO idempotency (
            user_id,
            idempotency_key,
            created_at
        )
        VALUES (
            $1, $2, now()
        )
        ON CONFLICT DO NOTHING
        "#,
        user_id,
        idempotency_key
    )
    .execute(pg_executor)
    .await
}

pub async fn pg_insert_newsletter_issue(
    pg_executor: impl PgExecutor<'_>,
    title: &str,
    text_content: &str,
    html_content: &str,
) -> Result<Uuid, sqlx::Error> {
    let newsletter_issue_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO newsletter_issues (
        newsletter_issue_id,
        title,
        text_content,
        html_content,
        published_at
        )
        VALUES ($1, $2, $3, $4, now())
        "#,
        newsletter_issue_id,
        title,
        text_content,
        html_content,
    )
    .execute(pg_executor)
    .await?;

    Ok(newsletter_issue_id)
}

pub async fn pg_enqueue_delivery_tasks(
    pg_executor: impl PgExecutor<'_>,
    newsletter_issue_id: Uuid,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO issue_delivery_queue (
        newsletter_issue_id,
        subscriber_email
        )
        SELECT $1, email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
        newsletter_issue_id
    )
    .execute(pg_executor)
    .await
}

#[tracing::instrument(skip_all)]
pub async fn pg_dequeue_task(
    pg_executor: impl PgExecutor<'_>,
) -> Result<Option<(Uuid, String)>, sqlx::Error> {
    let r = sqlx::query!(
        r#"
        SELECT newsletter_issue_id, subscriber_email
        FROM issue_delivery_queue
        FOR UPDATE
        SKIP LOCKED
        LIMIT 1
        "#,
    )
    .fetch_optional(pg_executor)
    .await?;

    match r {
        Some(r) => Ok(Some((r.newsletter_issue_id, r.subscriber_email))),
        None => Ok(None),
    }
}

pub async fn pg_delete_task(
    pg_executor: impl PgExecutor<'_>,
    issue_id: Uuid,
    email: &str,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query!(
        r#"
        DELETE FROM issue_delivery_queue
        WHERE
            newsletter_issue_id = $1
            AND
            subscriber_email = $2
        "#,
        issue_id,
        email
    )
    .execute(pg_executor)
    .await
}

pub async fn pg_get_issue(
    pg_executor: impl PgExecutor<'_>,
    issue_id: Uuid,
) -> Result<NewsletterIssue, sqlx::Error> {
    sqlx::query_as!(
        NewsletterIssue,
        r#"
        SELECT title, text_content, html_content
        FROM newsletter_issues
        WHERE
            newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(pg_executor)
    .await
}
