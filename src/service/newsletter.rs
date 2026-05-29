use anyhow::Context;
use axum::{
    http,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    database::postgres::{
        idempotency::{get_idempotency_response, save_idempotency_key, save_idempotency_response},
        subscriptions::get_confirmed_subscribers,
    },
    domain::{
        extractor::TokenData,
        form_data::PostNewsletterFormData,
        idempotency::{IdempotencyKey, SavedIdempotencyResponse},
        subscriber_email::SubscriberEmail,
    },
    service::error::ServiceError,
};

pub struct NewsletterService;

impl NewsletterService {
    pub async fn publish_newsletter(
        &self,
        app_state: &AppState,
        email: &PostNewsletterFormData,
    ) -> Result<(), ServiceError> {
        let confirmed_subscribers = get_confirmed_subscribers(&app_state.pg_pool).await?;
        let subscribers = confirmed_subscribers.into_iter().filter_map(|r| {
            SubscriberEmail::parse(r.email)
                .map_err(|e| {
                    tracing::warn!(
                        error = ?e,
                        "Skipping a confirmed subscriber. Their stored contact details are invalid",
                    )
                })
                .ok()
        });

        for subscriber in subscribers {
            app_state
                .email_client
                .send_email(
                    &subscriber,
                    &email.title,
                    &email.html_content,
                    &email.text_content,
                )
                .await
                .inspect_err(|e| {
                    tracing::error!(e = ?e, "Failed to send newsletter issue to {}", subscriber);
                })?;
        }

        Ok(())
    }

    /// 저장한 응답이 있는지 확인한다.
    pub async fn get_idempotency_response(
        &self,
        app_state: &AppState,
        email: &PostNewsletterFormData,
        token_data: &TokenData,
    ) -> Result<Option<SavedIdempotencyResponse>, ServiceError> {
        let idempotency_key = IdempotencyKey(email.idempotency_key);
        let saved_response =
            get_idempotency_response(&app_state.pg_pool, &idempotency_key, &token_data.user_id)
                .await?;

        Ok(saved_response)
    }

    /// 응답을 저장한다
    pub async fn save_idempotency_response(
        &self,
        app_state: &AppState,
        idempotency_key: &IdempotencyKey,
        user_id: &Uuid,
        response: Response,
    ) -> Result<SavedIdempotencyResponse, ServiceError> {
        let saved_response = SavedIdempotencyResponse::extract_response(response)
            .await
            .context("cannot convert Response to SavedIdempotencyResponse")
            .map_err(ServiceError::UnexpectedError)?;

        save_idempotency_response(
            &app_state.pg_pool,
            idempotency_key,
            user_id,
            &saved_response,
        )
        .await?;

        Ok(saved_response)
    }

    pub async fn try_processing(
        &self,
        app_state: &AppState,
        token_data: &TokenData,
        post_newsletter_form_data: &PostNewsletterFormData,
    ) -> Result<SavedIdempotencyResponse, ServiceError> {
        let idempotency_key = IdempotencyKey(post_newsletter_form_data.idempotency_key);
        let mut transaction = app_state.pg_pool.begin().await?;

        let response =
            match save_idempotency_key(transaction.as_mut(), &idempotency_key, &token_data.user_id)
                .await?
            {
                Some(_) => {
                    app_state
                        .newsletter_service
                        .publish_newsletter(app_state, post_newsletter_form_data)
                        .await?;
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
}
