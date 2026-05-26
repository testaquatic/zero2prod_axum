use sqlx::PgPool;

use crate::{
    database::postgres::newsletters::get_confirmed_subscribers,
    domain::subscriber_email::SubscriberEmail, email_client::EmailClient,
    handler::newsletter::BodyData, service::error::ServiceError,
};

pub struct NewsletterService;

impl NewsletterService {
    pub async fn publish_newsletter(
        &self,
        pg_pool: &PgPool,
        email: &BodyData,
        email_client: &EmailClient,
    ) -> Result<(), ServiceError> {
        let confirmed_subscribers = get_confirmed_subscribers(pg_pool).await?;
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
            email_client
                .send_email(
                    &subscriber,
                    &email.title,
                    &email.content.html,
                    &email.content.text,
                )
                .await
                .inspect_err(|e| {
                    tracing::error!(e = ?e, "Failed to send newsletter issue to {}", subscriber);
                })?;
        }

        Ok(())
    }
}
