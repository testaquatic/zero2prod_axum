use axum::http::{HeaderName, HeaderValue};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::domain::idempotency::{IdempotencyKey, SavedIdempotencyResponse};

pub async fn get_idempotency_data(
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
        response_headers: row
            .response_headers
            .into_iter()
            .map(|header_pair| (header_pair.name, header_pair.value))
            .collect(),
        response_body: row.response_body,
    });

    Ok(saved_response)
}

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "header_pair")]
struct HeaderPairRecord {
    name: String,
    value: Vec<u8>,
}

impl TryFrom<HeaderPairRecord> for (HeaderName, HeaderValue) {
    type Error = anyhow::Error;
    fn try_from(value: HeaderPairRecord) -> Result<Self, Self::Error> {
        Ok((value.name.parse()?, HeaderValue::from_bytes(&value.value)?))
    }
}
