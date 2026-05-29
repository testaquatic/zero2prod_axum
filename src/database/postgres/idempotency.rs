use sqlx::PgExecutor;
use uuid::Uuid;

use crate::domain::idempotency::{HeaderPairRecord, IdempotencyKey, SavedIdempotencyResponse};

#[tracing::instrument(name = "Get idempotency response", skip_all, err(Debug))]
pub async fn get_idempotency_response(
    pg_excutor: impl PgExecutor<'_>,
    idempotency_key: &IdempotencyKey,
    user_id: &Uuid,
) -> Result<Option<SavedIdempotencyResponse>, sqlx::Error> {
    let saved_response = sqlx::query!(
        r#"
        SELECT 
          response_status_code AS "response_status_code!",
          response_headers AS "response_headers!: Vec<HeaderPairRecord>", 
          response_body AS "response_body!"
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

#[tracing::instrument(name = "Save idempotency response", skip_all, err(Debug))]
pub async fn save_idempotency_response(
    pg_excutor: impl PgExecutor<'_>,
    idempotency_key: &IdempotencyKey,
    user_id: &Uuid,
    response: &SavedIdempotencyResponse,
) -> Result<(), sqlx::Error> {
    sqlx::query_unchecked!(
        r#"
        UPDATE idempotency
        SET response_status_code = $1, response_headers = $2, response_body = $3
        WHERE user_id = $4 AND idempotency_key = $5
        "#,
        response.response_status_code,
        response.response_headers,
        response.response_body,
        user_id,
        idempotency_key.as_ref(),
    )
    .execute(pg_excutor)
    .await
    .map(|_| ())
}

/// 키 저장에 성공하면 Ok(Some(&IdempotencyKey))를 반환
/// 실패하면 Ok(None)을 반환한다.
#[tracing::instrument(name = "Save idempotency key", skip_all, err(Debug))]
pub async fn save_idempotency_key<'a>(
    pg_excutor: impl PgExecutor<'_>,
    idempotency_key: &'a IdempotencyKey,
    user_id: &Uuid,
) -> Result<Option<&'a IdempotencyKey>, sqlx::Error> {
    let n_inserted_rows = sqlx::query!(
        r#"
        INSERT INTO idempotency (user_id, idempotency_key, created_at)
        VALUES ($1, $2, now())
        ON CONFLICT DO NOTHING
        "#,
        user_id,
        idempotency_key.as_ref()
    )
    .execute(pg_excutor)
    .await?
    .rows_affected();

    if n_inserted_rows == 0 {
        Ok(None)
    } else {
        Ok(Some(idempotency_key))
    }
}
