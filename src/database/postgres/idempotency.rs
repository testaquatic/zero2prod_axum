use sqlx::PgExecutor;
use uuid::Uuid;

use crate::domain::idempotency::{HeaderPairRecord, IdempotencyKey, SavedIdempotencyResponse};

pub async fn get_idempotency_response(
    pg_excutor: impl PgExecutor<'_>,
    idempotency_key: &IdempotencyKey,
    user_id: &Uuid,
) -> Result<Option<SavedIdempotencyResponse>, sqlx::Error> {
    let saved_response = sqlx::query!(
        r#"
        SELECT 
          response_status_code, 
          response_headers AS "response_headers: Vec<HeaderPairRecord>", 
          response_body
        FROM idempotency
        WHERE user_id = $1 AND idempotency_key = $2
        "#,
        user_id,
        idempotency_key.as_ref()
    )
    .fetch_optional(pg_excutor)
    .await?
    .map(|row| SavedIdempotencyResponse {
        response_status_code: row.response_status_code,
        response_headers: row.response_headers,
        response_body: row.response_body,
    });

    Ok(saved_response)
}

pub async fn save_idempotency_response(
    pg_excutor: impl PgExecutor<'_>,
    idempotency_key: &IdempotencyKey,
    user_id: &Uuid,
    response: &SavedIdempotencyResponse,
) -> Result<(), sqlx::Error> {
    sqlx::query_unchecked!(
        r#"
        INSERT INTO idempotency (
          user_id, 
          idempotency_key, 
          response_status_code, 
          response_headers, 
          response_body, 
          created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        user_id,
        idempotency_key.0,
        response.response_status_code,
        response.response_headers,
        response.response_body,
        chrono::Utc::now(),
    )
    .execute(pg_excutor)
    .await
    .map(|_| ())
}
