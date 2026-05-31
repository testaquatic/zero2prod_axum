use anyhow::Context;
use axum::{http, response::IntoResponse};
use sqlx::PgPool;
use tracing::{Span, field::display};

use crate::{
    app_state::AppState,
    database::postgres::{
        idempotency::{get_idempotency_response, save_idempotency_key, save_idempotency_response},
        issue_delivery_queue::{delete_task, dequeue_task, enqueue_delivery_task},
        newslettter_issues::{get_issue, save_newsletter_issue},
    },
    domain::{
        extractor::TokenData,
        form_data::PostNewsletterData,
        idempotency::{IdempotencyKey, SavedIdempotencyResponse},
        subscriber_email::SubscriberEmail,
    },
    email_client::EmailClient,
    service::error::ServiceError,
};

pub struct NewsletterService;

impl NewsletterService {
    pub async fn try_processing(
        &self,
        app_state: &AppState,
        token_data: &TokenData,
        post_newsletter_data: &PostNewsletterData,
    ) -> Result<SavedIdempotencyResponse, ServiceError> {
        let idempotency_key = IdempotencyKey(post_newsletter_data.idempotency_key);
        let mut transaction = app_state.pg_pool.begin().await?;

        let response =
            match save_idempotency_key(transaction.as_mut(), &idempotency_key, &token_data.user_id)
                .await?
            {
                Some(_) => {
                    let issue_id =
                        save_newsletter_issue(transaction.as_mut(), post_newsletter_data).await?;
                    enqueue_delivery_task(transaction.as_mut(), issue_id).await?;

                    let response = http::StatusCode::OK.into_response();
                    let response = SavedIdempotencyResponse::extract_response(response)
                        .await
                        .context("cannot convert Response to SavedIdempotencyResponse")
                        .map_err(ServiceError::UnexpectedError)?;
                    save_idempotency_response(
                        transaction.as_mut(),
                        &idempotency_key,
                        &token_data.user_id,
                        &response,
                    )
                    .await?;

                    Ok(response)
                }
                None => {
                    let saved_response = get_idempotency_response(
                        transaction.as_mut(),
                        &idempotency_key,
                        &token_data.user_id,
                    )
                    .await?
                    .context("We expected a saved response, we didn't find it")
                    .map_err(ServiceError::UnexpectedError)?;

                    Ok(saved_response)
                }
            };

        transaction.commit().await?;

        response
    }

    /// 치명적인 오류가 아니라면 오류를 반환하지 않고 진행한다.
    /// 전송한 이메일을 반환한다.
    #[tracing::instrument(skip_all, err(Debug), fields(newsletter_issue_id = tracing::field::Empty, subscriber_email = tracing::field::Empty))]
    pub async fn try_execute_task(
        &self,
        email_client: &EmailClient,
        pool: &PgPool,
    ) -> Result<Option<SubscriberEmail>, ServiceError> {
        let mut transaction = pool.begin().await?;
        if let Some((issue_id, subscriber_id, email)) = dequeue_task(transaction.as_mut()).await? {
            delete_task(transaction.as_mut(), issue_id, subscriber_id).await?;
            transaction.commit().await?;

            Span::current()
                .record("newsletter_issue_id", display(&issue_id))
                .record("subscriber_email", display(&email));
            let Ok(email) = SubscriberEmail::parse(email).inspect_err(|e| {
                tracing::error!(
                    error.cause_chain = ?e,
                    error.message = %e,
                    "Skipping a confirmed subscriber. Their stored contact details are invalid",
                )
            }) else {
                return Ok(None);
            };

            let issue = get_issue(pool, issue_id).await?;

            let _ = email_client
                .send_email(
                    &email,
                    &issue.title,
                    &issue.html_content,
                    &issue.text_content,
                )
                .await
                .inspect_err(|e| {
                    tracing::error!(
                        error.cause_chain = ?e,
                        error.message = %e,
                        "Failed to send deliver issue to a confirmed subscriber. Skipping",
                    )
                });

            return Ok(Some(email));
        }

        Ok(None)
    }
}
