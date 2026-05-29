use crate::{
    app_state::AppState,
    database::postgres::{
        idempotency::get_idempotency_data, subscriptions::get_confirmed_subscribers,
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
            get_idempotency_data(&app_state.pg_pool, &idempotency_key, &token_data.user_id).await?;

        Ok(saved_response)
    }
}
