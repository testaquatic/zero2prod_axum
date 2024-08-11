use axum::{
    body::to_bytes,
    response::{IntoResponse, Response},
};
use sqlx::{postgres::PgQueryResult, Postgres, Transaction};
use uuid::Uuid;

use crate::database::{types::HeaderPairRecord, Z2PADBError};

use super::postgres_query::{
    pg_delete_task, pg_enqueue_delivery_tasks, pg_insert_newsletter_issue, pg_save_response,
};

pub struct PostgresTransaction<'a> {
    pg_transaction: Transaction<'a, Postgres>,
}

impl<'a> AsMut<Transaction<'a, Postgres>> for PostgresTransaction<'a> {
    fn as_mut(&mut self) -> &mut Transaction<'a, Postgres> {
        &mut self.pg_transaction
    }
}

impl<'a> PostgresTransaction<'a> {
    pub fn new(pg_transaction: Transaction<'a, Postgres>) -> Self {
        Self { pg_transaction }
    }

    // `PostgresTransaction`의 소유권을 가져가서 다시 사용하지 못하게 한다.
    #[tracing::instrument(skip_all)]
    pub async fn commit(self) -> Result<(), Z2PADBError> {
        self.pg_transaction
            .commit()
            .await
            .map_err(Z2PADBError::SqlxError)
    }

    #[tracing::instrument(skip_all)]
    async fn enqueue_delivery_tasks(
        &mut self,
        newsletter_issue_id: Uuid,
    ) -> Result<PgQueryResult, Z2PADBError> {
        pg_enqueue_delivery_tasks(self.pg_transaction.as_mut(), newsletter_issue_id)
            .await
            .map_err(Z2PADBError::SqlxError)
    }

    async fn insert_newsletter_issue(
        &mut self,
        title: &str,
        text_content: &str,
        html_content: &str,
    ) -> Result<Uuid, Z2PADBError> {
        let uuid = pg_insert_newsletter_issue(
            self.pg_transaction.as_mut(),
            title,
            text_content,
            html_content,
        )
        .await?;

        Ok(uuid)
    }

    pub async fn save_response(
        mut self,
        idempotency_key: &crate::idempotency::IdempotencyKey,
        user_id: Uuid,
        http_response: axum::response::Response,
    ) -> Result<Response, Z2PADBError> {
        let (header, body) = http_response.into_parts();
        let status_code = header.status.as_u16() as i16;
        let headers = header
            .headers
            .iter()
            .map(|(name, value)| HeaderPairRecord {
                name: name.to_string(),
                value: value.as_bytes().to_vec(),
            })
            .collect::<Vec<_>>();
        let body = to_bytes(body, usize::MAX)
            .await
            .map_err(Z2PADBError::AzumCoreError)?
            .to_vec();

        pg_save_response(
            self.pg_transaction.as_mut(),
            user_id,
            idempotency_key.as_ref(),
            status_code,
            &headers,
            &body,
        )
        .await?;
        self.commit().await?;

        Ok((header, body).into_response())
    }

    pub async fn schedule_newsletter_delivery(
        &mut self,
        title: &str,
        text_content: &str,
        html_content: &str,
    ) -> Result<(), Z2PADBError> {
        let issue_id = self
            .insert_newsletter_issue(title, text_content, html_content)
            .await?;

        self.enqueue_delivery_tasks(issue_id).await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn delete_task(
        mut self,
        issue_id: Uuid,
        email: &str,
    ) -> Result<PgQueryResult, Z2PADBError> {
        let pg_query_result = pg_delete_task(self.as_mut().as_mut(), issue_id, email)
            .await
            .map_err(Z2PADBError::SqlxError)?;
        self.commit().await?;

        Ok(pg_query_result)
    }
}
