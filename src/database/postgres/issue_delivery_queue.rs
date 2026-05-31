use sqlx::PgExecutor;
use uuid::Uuid;

#[tracing::instrument(name = "Enqueue delivery task", skip_all, err(Debug))]
pub async fn enqueue_delivery_task(
    pg_excutor: impl PgExecutor<'_>,
    newsletter_issue_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO issue_delivery_queue (
            newsletter_issue_id,
            subscriber_id
        ) SELECT $1, id
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
        newsletter_issue_id
    )
    .execute(pg_excutor)
    .await
    .map(|_| ())
}

/// `Option<(newsletter_issue_id, subscriber_id, email)>`를 반환
#[tracing::instrument(name = "Dequeue delivery task", skip_all, err(Debug))]
pub async fn dequeue_task(
    pg_excutor: impl PgExecutor<'_>,
) -> Result<Option<(Uuid, Uuid, String)>, sqlx::Error> {
    let result = sqlx::query!(
        r#" 
        WITH queue AS (
          SELECT newsletter_issue_id, subscriber_id
          FROM issue_delivery_queue
          FOR UPDATE SKIP LOCKED
          LIMIT 1
        ) 
         SELECT queue.newsletter_issue_id, queue.subscriber_id, email
         FROM subscriptions
         JOIN queue ON queue.subscriber_id = subscriptions.id
         WHERE subscriptions.id = queue.subscriber_id
        "#,
    )
    .fetch_optional(pg_excutor)
    .await?
    .map(|r| (r.newsletter_issue_id, r.subscriber_id, r.email));

    Ok(result)
}

#[tracing::instrument(name = "Delete delivery task", skip_all, err(Debug))]
pub async fn delete_task(
    pg_excutor: impl PgExecutor<'_>,
    newsletter_issue_id: Uuid,
    subscriber_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        DELETE FROM issue_delivery_queue
        WHERE newsletter_issue_id = $1 AND subscriber_id = $2
        "#,
        newsletter_issue_id,
        subscriber_id
    )
    .execute(pg_excutor)
    .await
    .map(|_| ())
}
